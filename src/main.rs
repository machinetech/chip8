mod chip8;
 
extern crate sdl2;
extern crate time;

use chip8::{GFX_H,GFX_W};
use chip8::emu::Emu;
use chip8::ui::Ui;
use chip8::metro::Metronome;
use sdl2::event::Event;
use sdl2::keycode::KeyCode;
use std::env;
use std::io::Read;
use std::path::Path;
use std::fs::File;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

// Load the emulator with the indicated ROM. 
fn load_rom(emu: &mut Emu , path_to_rom: &Path) { 
    let mut file = File::open(&path_to_rom).unwrap();
    let mut rom: Vec<u8> = Vec::new();
    file.read_to_end(&mut rom).unwrap();
    emu.load_rom(rom);
}

// Messages that get passed from the ui to the emulator.
enum UiToEmuMsg { Keys([bool; 16]), Paused(bool), Quit, Reset }

// Messages that get passed from the emulator to the ui.
enum EmuToUiMsg { Beeping(bool), Draw([[bool; GFX_H]; GFX_W]), QuitAck }

// Drives user interaction. Responsible for processing keypresses, updating
// the screen and playing audible beeps. Communicates with the emulator by
// exchanging messages across a two way channel. 
//
// Runs on the main thread.
// Allows the UI to be agnostic of the emulator (and the channels by
// which we communicate with the emulator).
//
// Arrows indicate knowledge, not data flow:
// ui <- ui_exec -> channel <- emu_exec -> emmulator
// 
// Arrows indicate data flow: 
// ui <-> ui_exec <-> channel <-> emu_exec <-> emmulator
//
fn ui_exec(mut ui: Ui, tx: Sender<UiToEmuMsg>, rx: Receiver<EmuToUiMsg>) {
    let mut refresh_gfx_rate = Metronome::new(120);
    let mut paused = false;
    'ui_exec_loop: loop {
        poll_key_presses(&mut ui, &tx, &mut paused); 
        if poll_emu_events(&mut ui, &rx, &paused, &mut refresh_gfx_rate) {
            break 'ui_exec_loop;
        }
        // Short sleep to free up cpu cycles
        thread::sleep_ms(1);    
    }
}

// Poll for and handle key press events. 
fn poll_key_presses(ui: &mut Ui, tx: &Sender<UiToEmuMsg>, 
                    paused: &mut bool) {
    match ui.sdl_ctx.event_pump().poll_event() {
        None => {},
        Some(event) => {
            match event {
                Event::Quit{..} => {
                    tx.send(UiToEmuMsg::Paused(*paused)).unwrap(); 
                },
                Event::KeyDown{keycode,..} => match keycode {
                    KeyCode::Escape => {
                        // Signal emulator with intention to quit
                        // and allow it to shutdown gracefully.
                        tx.send(UiToEmuMsg::Quit).unwrap(); 
                    },
                    KeyCode::Return => {
                        // Signal emulator to pause.
                        *paused ^= true; 
                        tx.send(UiToEmuMsg::Paused(*paused)).unwrap();
                    },
                    KeyCode::Backspace => {
                        // Signal emulator to reset.
                        tx.send(UiToEmuMsg::Reset).unwrap();
                        *paused = false;
                        tx.send(UiToEmuMsg::Paused(*paused)).unwrap();
                    },
                    _ => if !*paused {
                        // A key was pressed, signal emulator with updated
                        // key states.
                        tx.send(UiToEmuMsg::Keys(
                                Ui::get_updated_keys())).unwrap();
                    }, 
                },
                Event::KeyUp{..} => if !*paused {
                    // A key was released, signal emulator with updated
                    // key states.
                    tx.send(UiToEmuMsg::Keys(
                            Ui::get_updated_keys())).unwrap();
                },
                _ => {}
            }
        }
    }
}

// Poll for and handle emulator events. Returns true if emulator acknowledged 
// quit. 
fn poll_emu_events(ui: &mut Ui, rx: &Receiver<EmuToUiMsg>, paused: &bool, 
                   refresh_gfx_rate: &mut Metronome) -> bool {
    match rx.try_recv() {
        Ok(emu_event) => {
            match emu_event {
                // Handle beeb state change signalled by emulator.
                EmuToUiMsg::Beeping(on) => ui.beep(on),
                // Handle draw event signalled by emulator.
                EmuToUiMsg::Draw(ref gfx) => {
                    refresh_gfx_rate.on_tick(|| {
                        if !*paused { ui.refresh_gfx(gfx); }
                    });
                },
                // Emulator has acknowledged the earlier quit signal.
                // The ui thread may shutdown in response.
                EmuToUiMsg::QuitAck => return true,
            }
        },
        _ => {},
    } 
    false
}

// Drives the emulator. Communicates with the user interface by exchanging
// messages across a two way channel. 
//
// Assigned its own thread. 
// Allows the emulator to be agnostic of the ui (and the channels by
// which we communicate with the ui).
//
// Arrows indicate knowledge, not data flow:
// ui <- ui_exec -> channel <- emu_exec -> emmulator
// 
// Arrows indicate data flow: 
// ui <-> ui_exec <-> channel <-> emu_exec <-> emmulator
//
fn emu_exec(mut emu: Emu, tx:Sender<EmuToUiMsg>, rx: Receiver<UiToEmuMsg>) {
    let mut clock_rate = Metronome::new(500);
    let mut update_timers_rate = Metronome::new(60);
    let mut paused = false;
    let mut beeping = false;
    
    'emu_exec_loop: loop {
        // Poll for ui events.
        match rx.try_recv() {
            Ok(ui_to_emu_msg) => 
                match ui_to_emu_msg {
                    // New key press states.
                    UiToEmuMsg::Keys(new_keys) => emu.keys = new_keys,
                    // Reset everything.
                    UiToEmuMsg::Reset => emu.reset(),
                    // Pause or unpause.
                    UiToEmuMsg::Paused(p) => paused = p,
                    // Acknowledge quit and shut down gracefully.
                    UiToEmuMsg::Quit => {
                        tx.send(EmuToUiMsg::QuitAck).unwrap();
                        break 'emu_exec_loop;
                    }, 
                },
            _ => {},
        }  
       
        // Signal ui with draw event.
        clock_rate.on_tick(|| {
            if !paused {
                &mut emu.execute_cycle();
                if emu.draw {
                    tx.send(EmuToUiMsg::Draw(emu.gfx)).unwrap();
                    emu.draw = false;
                }
             } 
        });
        
        // Update emulator timers.
        update_timers_rate.on_tick(|| {
            if !paused { 
                emu.update_timers(); 
                if beeping != emu.beeping() {
                    beeping ^= true; 
                    // Signal ui with new beep state.
                    tx.send(EmuToUiMsg::Beeping(beeping)).unwrap();
                }
            }                
        });

        // Short sleep to free up cpu cycles
        thread::sleep_ms(1);    
    }
}

// Entry point into the program. Takes care of basic setup such as reading
// the ROM path from the command line and kicking off the ui and emulator.
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        print!("Usage: chip8 PATH_TO_ROM");
        return;
    }
    let path_to_rom = Path::new(&args[1]);
    let ui = Ui::new();
    let mut emu = Emu::new();
    load_rom(&mut emu, path_to_rom);
    // The channels through which the ui and emulator will communicate.
    let (tx1, rx1) = mpsc::channel::<UiToEmuMsg>();
    let (tx2, rx2) = mpsc::channel::<EmuToUiMsg>();
    // The emulator run in its own thread.
    thread::spawn(move || { 
        emu_exec(emu, tx2, rx1); 
    });
    // The ui runs on the main thread.
    ui_exec(ui, tx1, rx2);
}
