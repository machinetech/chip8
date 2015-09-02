mod chip8;
 
extern crate sdl2;
extern crate time;

use chip8::{GFX_H,GFX_W,Mode};
use chip8::emu::Emu;
use chip8::ui::Ui;
use chip8::metro::Metronome;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::env;
use std::io::Read;
use std::path::Path;
use std::fs::File;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;

// Load the emulator with the indicated ROM. 
fn load_rom(emu: &mut Emu, path_to_rom: &Path) { 
    let mut file = File::open(&path_to_rom).unwrap();
    let mut rom: Vec<u8> = Vec::new();
    file.read_to_end(&mut rom).unwrap();
    emu.load_rom(rom);
}

// Messages that get passed from the ui to the emulator.
enum UiToEmuMsg { Keys([bool; 16]), Paused(bool), Quit, Reset }

// Messages that get passed from the emulator to the ui.
enum EmuToUiMsg { Beeping(bool), Draw(Mode, [[bool; GFX_H]; GFX_W]), QuitAck }

// Drives user interaction. Responsible for processing keypresses, updating
// the screen and playing audible beeps. Communicates with the emulator by
// exchanging messages across a two way channel. 
//
// Runs on the main thread.
fn ui_exec(mut ui: Ui, tx: Sender<UiToEmuMsg>, rx: Receiver<EmuToUiMsg>) {
    let mut refresh_gfx_rate = Metronome::new(120);
    let mut paused = false;
    'ui_exec_loop: loop {
        process_key_presses(&mut ui, &tx, &mut paused); 
        if process_emu_events(&mut ui, &rx, &paused, &mut refresh_gfx_rate) {
            break 'ui_exec_loop;
        }
        // Short sleep to free up cpu cycles
        thread::sleep_ms(1);    
    }
}

// Poll for and handle key press events. 
fn process_key_presses(ui: &mut Ui, tx: &Sender<UiToEmuMsg>, 
                    paused: &mut bool) {
    match ui.poll_event() {
        None => {},
        Some(event) => {
            match event {
                Event::Quit{..} => {
                    tx.send(UiToEmuMsg::Paused(*paused)).unwrap(); 
                },
                Event::KeyDown{keycode,..} => match keycode {
                    Option::Some(Keycode::Escape) => {
                        // Signal emulator with intention to quit
                        // and allow it to shutdown gracefully.
                        tx.send(UiToEmuMsg::Quit).unwrap(); 
                    },
                    Option::Some(Keycode::Return) => {
                        // Signal emulator to pause.
                        *paused ^= true; 
                        tx.send(UiToEmuMsg::Paused(*paused)).unwrap();
                    },
                    Option::Some(Keycode::Backspace) => {
                        // Signal emulator to reset.
                        tx.send(UiToEmuMsg::Reset).unwrap();
                        *paused = false;
                        tx.send(UiToEmuMsg::Paused(*paused)).unwrap();
                    },
                    _ => if !*paused {
                        // A key was pressed, signal emulator with updated
                        // key states.
                        tx.send(UiToEmuMsg::Keys(
                                ui.get_updated_keys())).unwrap();
                    }, 
                },
                Event::KeyUp{..} => if !*paused {
                    // A key was released, signal emulator with updated
                    // key states.
                    tx.send(UiToEmuMsg::Keys(
                            ui.get_updated_keys())).unwrap();
                },
                _ => {}
            }
        }
    }
}

// Poll for and handle emulator events. Returns true if emulator acknowledged 
// earlier quit signal. 
fn process_emu_events(ui: &mut Ui, rx: &Receiver<EmuToUiMsg>, paused: &bool, 
                      refresh_gfx_rate: &mut Metronome) -> bool {
    match rx.try_recv() {
        Ok(emu_event) => {
            match emu_event {
                // Handle beeb state change signalled by emulator.
                EmuToUiMsg::Beeping(on) => ui.beep(on),
                // Handle draw event signalled by emulator.
                EmuToUiMsg::Draw(ref mode, ref gfx) => {
                    refresh_gfx_rate.on_tick(|| {
                        if !*paused { ui.refresh_gfx(*mode, gfx); }
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
fn emu_exec(mut emu: Emu, tx: Sender<EmuToUiMsg>, rx: Receiver<UiToEmuMsg>) {
    let mut clock_rate = Metronome::new(500);
    let mut update_timers_rate = Metronome::new(60);
    let mut paused = false;
    let mut beeping = false;
    'emu_exec_loop: loop {
        if process_ui_events(&mut emu, &tx, &rx, &mut paused) {
            break 'emu_exec_loop;
        }
        signal_draw_event(&mut emu, &tx, &paused, &mut clock_rate); 
        update_timers(&mut emu, &tx, &paused, &mut beeping, 
                      &mut update_timers_rate);
        // Short sleep to free up cpu cycles
        thread::sleep_ms(1);    
    }
}

// Poll for and handle UI events. Returns true if Quit signal received from UI.
fn process_ui_events(emu: &mut Emu, tx: &Sender<EmuToUiMsg>,  
                     rx: &Receiver<UiToEmuMsg>, paused: &mut bool) -> bool {
    match rx.try_recv() {
        Ok(ui_to_emu_msg) => 
            match ui_to_emu_msg {
                // New key press states.
                UiToEmuMsg::Keys(new_keys) => emu.keys = new_keys,
                // Reset everything.
                UiToEmuMsg::Reset => emu.reset(),
                // Pause or unpause.
                UiToEmuMsg::Paused(p) => *paused = p,
                // Acknowledge quit and shut down gracefully.
                UiToEmuMsg::Quit => {
                    tx.send(EmuToUiMsg::QuitAck).unwrap();
                    return true;
                }, 
            },
        _ => {},
    }  
    false
}

// Signal the ui with a draw event.
fn signal_draw_event(emu: &mut Emu, tx: &Sender<EmuToUiMsg>, paused: &bool,
                     clock_rate: &mut Metronome) {
    clock_rate.on_tick(|| {
        if !paused {
            &mut emu.execute_cycle();
            if emu.draw {
                tx.send(EmuToUiMsg::Draw(emu.mode, emu.gfx)).unwrap();
                emu.draw = false;
            }
         } 
    });
}

// Update the emulator timers and signal the ui if the beep state changed.
fn update_timers(emu: &mut Emu, tx: &Sender<EmuToUiMsg>, paused: &bool, 
                 beeping: &mut bool, update_timers_rate: &mut Metronome) {
    update_timers_rate.on_tick(|| {
        if !paused { 
            emu.update_timers(); 
            if *beeping != emu.beeping() {
                *beeping ^= true; 
                tx.send(EmuToUiMsg::Beeping(*beeping)).unwrap();
            }
        }                
    });
}

// Entry point into the program. Takes care of basic setup such as reading
// the rom path from the command line and kicking off the ui and emulator.
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
