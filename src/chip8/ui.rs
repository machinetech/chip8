use sdl2;
use sdl2::audio::{AudioCallback,AudioDevice,AudioSpecDesired};
use sdl2::event::Event;
use sdl2::pixels::Color::RGB;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::keyboard::Scancode;
use sdl2::Sdl;
use super::{GFX_H,GFX_W,Mode,wav};

const SCALE: usize = 8;

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
    sdl_ctx: Sdl,
    renderer: Renderer<'static>,
    audio: AudioDevice<BeepCallback>,
}

impl Ui {
    
    pub fn new() -> Self {
        let sdl_ctx = sdl2::init().unwrap();
        let video_subsystem = sdl_ctx.video().unwrap();
        let window = video_subsystem.window("chip8", 
                                     (GFX_W * SCALE) as u32, 
                                     (GFX_H * SCALE) as u32)
                                    .position_centered()
                                    .build()
                                    .unwrap();

        let renderer = window.renderer().build().unwrap(); 
        
        let audio_subsystem = sdl_ctx.audio().unwrap();
        let audio_spec = AudioSpecDesired {
            freq: Some(wav::SAMPLE_RATE_HZ as i32),
            channels: Some(wav::CHANNELS as u8),
            samples: Some(wav::SAMPLES as u16)
        };
    
        let audio = audio_subsystem.open_playback(None, audio_spec, |_| {
            BeepCallback::new()
        }).unwrap();

        Ui { sdl_ctx: sdl_ctx, renderer: renderer, audio: audio } 
    }

    pub fn beep(&self, on: bool) {
        match on {
            true => self.audio.resume(),
            false => self.audio.pause()
        }
    }

    pub fn refresh_gfx(&mut self, mode: Mode, gfx: &[[bool; GFX_H]; GFX_W]) {
        let bg = RGB(0x1c, 0x28, 0x41);
        let fg = RGB(0xff, 0xff, 0xff);
        let projection_factor = match (mode) { 
            //
            // For STANDARD mode, the 64x32 gfx subscreen will be projected 
            // to fit the entire viewable area. The excess between 64x32 and
            // 128x64 will be projected offscreen. 
            // +-----------------------+-----------------------+
            // |                       |                       |
            // |         64x32         |                       |
            // |                       |                       |
            // +-----------------------+    drawn offscreen    |
            // |                                               |
            // |                                               |
            // |                                               |
            // +-----------------------------------------------+ (128x64)
            Mode::STANDARD => SCALE * 2, 
            Mode::SUPER => SCALE 
        };
        for x in 0..GFX_W {
            for y in 0..GFX_H {
                let pix_on = gfx[x][y];
                let color = if pix_on {fg} else {bg};
                let rx = (x * projection_factor) as i32;
                let ry = (y * projection_factor) as i32;
                let rw = projection_factor as u32;
                let rh = projection_factor as u32;
                let rect = Rect::new(rx, ry, rw, rh).unwrap().unwrap();
                self.renderer.set_draw_color(color);
                self.renderer.fill_rect(rect);
            }
        }
        self.renderer.present();
    } 
    
    pub fn poll_event(&self) -> Option<Event> {
        let mut event_pump = self.sdl_ctx.event_pump().unwrap();
        return event_pump.poll_event();
    }

    pub fn get_updated_keys(&self) -> [bool; 16] {
        let event_pump = self.sdl_ctx.event_pump().unwrap();
        let keyboard_state = event_pump.keyboard_state();
        let mut keys = [false; 16];
        keys[0x0] = keyboard_state.is_scancode_pressed(Scancode::X);
        keys[0x1] = keyboard_state.is_scancode_pressed(Scancode::Num1);
        keys[0x2] = keyboard_state.is_scancode_pressed(Scancode::Num2);
        keys[0x3] = keyboard_state.is_scancode_pressed(Scancode::Num3);
        keys[0x4] = keyboard_state.is_scancode_pressed(Scancode::Q);
        keys[0x5] = keyboard_state.is_scancode_pressed(Scancode::W);
        keys[0x6] = keyboard_state.is_scancode_pressed(Scancode::E);
        keys[0x7] = keyboard_state.is_scancode_pressed(Scancode::A);
        keys[0x8] = keyboard_state.is_scancode_pressed(Scancode::S);
        keys[0x9] = keyboard_state.is_scancode_pressed(Scancode::D);
        keys[0xA] = keyboard_state.is_scancode_pressed(Scancode::Z);
        keys[0xB] = keyboard_state.is_scancode_pressed(Scancode::C);
        keys[0xC] = keyboard_state.is_scancode_pressed(Scancode::Num4);
        keys[0xD] = keyboard_state.is_scancode_pressed(Scancode::R);
        keys[0xE] = keyboard_state.is_scancode_pressed(Scancode::F);
        keys[0xF] = keyboard_state.is_scancode_pressed(Scancode::V);
        keys
    }

}
