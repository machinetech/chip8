use sdl2;
use sdl2::audio::{AudioCallback,AudioDevice,AudioSpecDesired};
use sdl2::keyboard;
use sdl2::keycode::KeyCode;
use sdl2::pixels::Color::RGB;
use sdl2::rect::Rect;
use sdl2::render::{ACCELERATED,RenderDriverIndex,Renderer};
use sdl2::scancode::ScanCode;
use sdl2::video::{self,Window,WindowPos};
use sdl2::Sdl;
use std::collections::HashMap;
use super::{GFX_H,GFX_W,wav};

const SCALE: usize = 16;

pub struct BeepCallback;

impl BeepCallback {
    fn new() -> Self {
        BeepCallback
    }
}

impl AudioCallback for BeepCallback {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        assert!(out.len() == wav::PLAYBACK_BUFFER.len());
        for i in 0..wav::PLAYBACK_BUFFER.len() {
            out[i] = wav::PLAYBACK_BUFFER[i];
        }
    }    

}

pub struct Ui {
    pub sdl_ctx: Sdl,
    pub renderer: Renderer<'static>,
    pub audio: AudioDevice<BeepCallback> 
}

impl Ui {
    
    pub fn new() -> Self {
        let sdl_ctx = sdl2::init(sdl2::INIT_EVERYTHING).unwrap();
        let window = Window::new(&sdl_ctx,
                                 "chip8", 
                                 WindowPos::PosCentered,
                                 WindowPos::PosCentered,
                                 (GFX_W * SCALE) as i32, 
                                 (GFX_H * SCALE) as i32,
                                 video::SHOWN).unwrap();

        let renderer = Renderer::from_window(window, RenderDriverIndex::Auto, 
                                             ACCELERATED).unwrap();

        let audio_spec = AudioSpecDesired {
            freq: Some(wav::SAMPLE_RATE_HZ as i32),
            channels: Some(wav::CHANNELS as u8),
            samples: Some(wav::SAMPLES as u16)
        };
    
        let audio = AudioDevice::open_playback(None, audio_spec, |_| {
            BeepCallback::new()
        }).unwrap();

        Ui {sdl_ctx: sdl_ctx, renderer: renderer, audio: audio} 
    }

    pub fn beep(&self, on: bool) {
        match on {
            true => self.audio.resume(),
            false => self.audio.pause()
        }
    }

    pub fn refresh_gfx(&mut self, gfx: &[[bool; GFX_H]; GFX_W]) {
        let bg = RGB(0x1c, 0x28, 0x41);
        let fg = RGB(0xff, 0xff, 0xff);
        let mut drawer = self.renderer.drawer();
        drawer.clear();
        for x in 0..GFX_W {
            for y in 0..GFX_H {
                let pix_on = gfx[x][y];
                let color = if pix_on {fg} else {bg};
                let rx = (x * SCALE) as i32;
                let ry = (y * SCALE) as i32;
                let rw = SCALE as i32;
                let rh = SCALE as i32;
                let rect = Rect::new(rx, ry, rw, rh);
                drawer.set_draw_color(color);
                drawer.fill_rect(rect);
            }
        }
        drawer.present();
    } 

    pub fn get_updated_keys() -> [bool; 16] {
        let keyboard_state = keyboard::get_keyboard_state();
        let mut keys = [false; 16];
        keys[0x0] = Ui::get_key_state(&keyboard_state, KeyCode::X); 
        keys[0x1] = Ui::get_key_state(&keyboard_state, KeyCode::Num1); 
        keys[0x2] = Ui::get_key_state(&keyboard_state, KeyCode::Num2); 
        keys[0x3] = Ui::get_key_state(&keyboard_state, KeyCode::Num3); 
        keys[0x4] = Ui::get_key_state(&keyboard_state, KeyCode::Q); 
        keys[0x5] = Ui::get_key_state(&keyboard_state, KeyCode::W); 
        keys[0x6] = Ui::get_key_state(&keyboard_state, KeyCode::E); 
        keys[0x7] = Ui::get_key_state(&keyboard_state, KeyCode::A); 
        keys[0x8] = Ui::get_key_state(&keyboard_state, KeyCode::S); 
        keys[0x9] = Ui::get_key_state(&keyboard_state, KeyCode::D); 
        keys[0xA] = Ui::get_key_state(&keyboard_state, KeyCode::Z); 
        keys[0xB] = Ui::get_key_state(&keyboard_state, KeyCode::C); 
        keys[0xC] = Ui::get_key_state(&keyboard_state, KeyCode::Num4); 
        keys[0xD] = Ui::get_key_state(&keyboard_state, KeyCode::R); 
        keys[0xE] = Ui::get_key_state(&keyboard_state, KeyCode::F); 
        keys[0xF] = Ui::get_key_state(&keyboard_state, KeyCode::V); 
        keys
    }

    fn get_key_state(kb_state: &HashMap<ScanCode, bool>, kc: KeyCode) -> bool {
        let sc = keyboard::get_scancode_from_key(kc);
        *kb_state.get(&sc).unwrap_or(&false)
    }
}
