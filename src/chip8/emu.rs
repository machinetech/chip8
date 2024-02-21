extern crate rand;

use super::{GFX_H,GFX_W,Mode};
use std::default::Default;
use std::cmp;
use std::mem;

const SMALL_GFX_W: usize = 64;
const SMALL_GFX_H: usize = 32;

const MAX_ROM_SIZE: usize = RAM_SIZE - PROGRAM_START;
const NUM_REGISTERS: usize = 16;
const PROGRAM_START: usize = 512; 
const RAM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const NUM_SUPER_MODE_RPL_FLAGS: usize = 8;

const FONT_MAP: [u8; 5 * 16] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
    0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
    0x90, 0x90, 0xf0, 0x10, 0x10, // 4
    0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
    0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
    0xf0, 0x10, 0x20, 0x40, 0x40, // 7
    0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
    0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
    0xf0, 0x90, 0xf0, 0x90, 0x90, // A
    0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
    0xf0, 0x80, 0x80, 0x80, 0xf0, // C
    0xe0, 0x90, 0x90, 0x90, 0xe0, // D
    0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
    0xf0, 0x80, 0xf0, 0x80, 0x80, // F
];

const SUPER_MODE_FONT_MAP: [u8; 10 * 16] = [
    0xff, 0xff, 0xc3, 0xc3, 0xc3, 0xc3, 0xc3, 0xc3, 0xff, 0xff, // 0
    0x18, 0x78, 0x78, 0x18, 0x18, 0x18, 0x18, 0x18, 0xff, 0xff, // 1
    0xff, 0xff, 0x03, 0x03, 0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, // 2
    0xff, 0xff, 0x03, 0x03, 0xff, 0xff, 0x03, 0x03, 0xff, 0xff, // 3
    0xc3, 0xc3, 0xc3, 0xc3, 0xff, 0xff, 0x03, 0x03, 0x03, 0x03, // 4
    0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, 0x03, 0x03, 0xff, 0xff, // 5
    0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, 0xc3, 0xc3, 0xff, 0xff, // 6
    0xff, 0xff, 0x03, 0x03, 0x06, 0x0c, 0x18, 0x18, 0x18, 0x18, // 7
    0xff, 0xff, 0xc3, 0xc3, 0xff, 0xff, 0xc3, 0xc3, 0xff, 0xff, // 8
    0xff, 0xff, 0xc3, 0xc3, 0xff, 0xff, 0x03, 0x03, 0xff, 0xff, // 9
    0x7e, 0xff, 0xc3, 0xc3, 0xc3, 0xff, 0xff, 0xc3, 0xc3, 0xc3, // A
    0xfc, 0xfc, 0xc3, 0xc3, 0xfc, 0xfc, 0xc3, 0xc3, 0xfc, 0xfc, // B
    0x3c, 0xff, 0xc3, 0xc0, 0xc0, 0xc0, 0xc0, 0xc3, 0xff, 0x3c, // C
    0xfc, 0xfe, 0xc3, 0xc3, 0xc3, 0xc3, 0xc3, 0xc3, 0xfe, 0xfc, // D
    0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, // E
    0xff, 0xff, 0xc0, 0xc0, 0xff, 0xff, 0xc0, 0xc0, 0xc0, 0xc0  // F
];

pub struct Emu {
    
    // Unlike a typical Intel processor, that uses little endian order for multi byte data types,
    // the CHIP8 processor uses big endian order. This is important when creating a multi byte 
    // types (such as u16) by combining individual bytes. For instance, when reading an opcode
    // by combining two bytes.
    //
    // For more on endiannes, see http://www.geeksforgeeks.org/little-and-big-endian-mystery/
    
    // Can run in one of two modes: STANDARD or SUPER.
    pub mode: Mode,
    // Graphics pixel is either set or not. 
    pub gfx: [[bool; GFX_H]; GFX_W], 
    // Maps state of keypresses. True means the key has been pressed.
    pub keys: [bool; 16],
    // Set when the graphics state has changed and requires a redraw.
    pub draw: bool,
    // The program instruction to execute. There are 35 opcodes in total,
    // each 2 bytes long. 
    opcode: u16,
    // There are 4,096 8-bit memory locations making for a total of 4KB RAM. 
    // +---------------------+= 0xfff=4095 
    // |                     |
    // |                     |
    // |                     |
    // | Program +           | 
    // | Data                | 
    // |                     |
    // |                     |
    // |                     |
    // |                     |
    // +---------------------+= 0x200=0512 
    // |                     | 
    // |                     | 
    // +---------------------+= 0x0f0=0240 
    // |                     | 
    // | SUPER_MODE_FONT_MAP |
    // |                     | 
    // +---------------------+= 0x050=0080 
    // |                     | 
    // | FONT_MAP            |
    // |                     | 
    // +---------------------+= 0x000=0000 
    //
    ram: [u8; RAM_SIZE],  
    // There are 16 8-bit registers, referred to as v0 to vf: v0 to vE are
    // general purpose while vf stores the carry flag.
    v: [u8; NUM_REGISTERS],            
    // The special purpose 16-bit index register is used to a memory address.
    // Only the lowest (rightmost) 12 bits are usually used.
    ram_idx: u16,                
    // The program counter is used to store the currently executing address.
    // a 'pseudo register' not directly accessible from programs.
    pc: u16,                
    // Special purpose 8-bit register for the delay timer. When value is non-
    // zero, then decremented at a rate of 60Hz.
    dt: u8,
    // Special purpose 8-bit register for the sound timer. When value is non-
    // zero, then decremented at a rate of 60Hz.
    st: u8,
    // Array of 16-bit values used to store the address that the interpreter 
    // should return to when finished with a subroutine. Support for 16 levels
    // of nested subroutines.
    stack: [u16; STACK_SIZE],
    // The stack pointer points to the next available slot in the stack!
    // stack[sp] <-- where next push will be placed
    // stack[sp-1] <-- top of the stack (where last entry pushed resides)
    // a 'pseudo register' not directly accessible from programs.
    sp: usize,
    // Super mode flags used by opcodes fx75 and fx85.
    super_mode_rpl_flags: [u8; NUM_SUPER_MODE_RPL_FLAGS],
    // We cache a copy of the rom to allow for convenient reset.
    rom: Vec<u8>
}

impl Default for Emu {
    
    fn default() -> Self {
        let mut emu = Emu {
            mode: Mode::STANDARD,
            opcode: 0,
            ram: [0; RAM_SIZE],  
            v: [0; NUM_REGISTERS],
            ram_idx: 0,                
            pc: PROGRAM_START as u16,                
            gfx: [[false; GFX_H]; GFX_W],
            dt: 0,
            st: 0,
            stack: [0; STACK_SIZE], 
            sp: 0, 
            keys: [false; 16],
            draw: false,
            super_mode_rpl_flags: [0; NUM_SUPER_MODE_RPL_FLAGS],
            rom: Vec::with_capacity(MAX_ROM_SIZE),
        };
        let mut i = 0;
        for j in 0..FONT_MAP.len() {
            emu.ram[i] = FONT_MAP[j];
            i += 1;
        }
        for k in 0..SUPER_MODE_FONT_MAP.len() {
            emu.ram[i] = SUPER_MODE_FONT_MAP[k];
            i += 1;
        }
        emu 
    }
}

impl Emu {

    // Create emulator.
    pub fn new() -> Self { 
        Default::default() 
    }
    
    // Load rom into emulator, but does not start execution. 
    pub fn load_rom(&mut self, rom: Vec<u8>) {
        if rom.len() > MAX_ROM_SIZE {
            panic!("Program too large to fit into memory");
        }
        self.rom = rom;
        for i in 0..self.rom.len() {
            self.ram[PROGRAM_START+i] = self.rom[i];
        }  
    }

    // Reset the program to the initial rom state.
    pub fn reset(&mut self) {
        let stale = mem::replace(self, Emu::new());
        self.load_rom(stale.rom);
    }

    // Perform a single fetch-decode-execute cycle.
    pub fn execute_cycle(&mut self) {
        self.fetch_opcode();
        self.decode_and_execute_opcode();
    }

    // Update the delay and sound timers.
    pub fn update_timers(&mut self) {
        if self.dt > 0 { self.dt -= 1; }
        if self.st > 0 { self.st -= 1; }
    }

    // Indicates whether the state justifies a beep at this
    // exact time.
    pub fn beeping(&self) -> bool {
        return self.st > 0;
    }
    
    // Return the gfx width.
    fn width(&self) -> usize {
        match self.mode {
            Mode::STANDARD => SMALL_GFX_W,
            Mode::SUPER => GFX_W
        }
    }

    // Return the gfx height.
    fn height(&self) -> usize {
        match self.mode {
            Mode::STANDARD => SMALL_GFX_H,
            Mode::SUPER => GFX_H
        }
    }
   
    // Scroll screen n lines down.
    fn execute_opcode_00cn(&mut self) {
        let n = (self.opcode & 0x000f) as usize; 
        for y in (n..GFX_H).rev() {
            for x in 0..GFX_W { self.gfx[x][y] = self.gfx[x][y-n]; }
        } 
        for y in 0..n {
            for x in 0..GFX_W { self.gfx[x][y] = false; }
        } 
        self.draw = true;
        self.pc = (self.pc + 2) & 0x0fff; 
    }  
    
    // Clear screen.
    fn execute_opcode_00e0(&mut self) {
        for x in 0..GFX_W { for y in 0..GFX_H { self.gfx[x][y] = false; } }
        self.draw = true;
        self.pc = (self.pc + 2) & 0x0fff; 
    }  
    
    // Return from last subroutine.
    fn execute_opcode_00ee(&mut self) {
        self.sp = (self.sp - 1) & (STACK_SIZE - 1); 
        self.pc = self.stack[self.sp] as u16; 
        self.pc = (self.pc + 2) & 0x0fff; 
    } 

    // Scroll screen 4 pixels right.
    fn execute_opcode_00fb(&mut self) {
        for y in 0..GFX_H {
            for x in (4..GFX_W).rev() { self.gfx[x][y] = self.gfx[x-4][y] }
            for x in 0..4 { self.gfx[x][y] = false; }
        }
        self.draw = true;
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Scroll screen 4 pixels left. 
    fn execute_opcode_00fc(&mut self) {
        for y in 0..GFX_H {
            for x in 0..(GFX_W - 4) { self.gfx[x][y] = self.gfx[x+4][y] }
            for x in (GFX_W-4)..GFX_W { self.gfx[x][y] = false; }
        }
        self.draw = true;
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Meant to exit, but we will reset instead.
    fn execute_opcode_00fd(&mut self) {
        self.reset();
    } 
    
    // Disable SUPER mode. 
    fn execute_opcode_00fe(&mut self) {
        self.mode = Mode::STANDARD;
        self.pc = (self.pc + 2) & 0x0fff; 
    } 
    
    // Enable SUPER mode. 
    fn execute_opcode_00ff(&mut self) {
        self.mode = Mode::SUPER;
        self.pc = (self.pc + 2) & 0x0fff; 
    } 
    
    // Jump to address nnn.
    fn execute_opcode_1nnn(&mut self) {
        let nnn = self.opcode & 0x0fff; 
        self.pc = nnn; 
    }

    // Call subroutine at nnn.
    fn execute_opcode_2nnn(&mut self) {
        let nnn = self.opcode & 0x0fff;
        self.stack[self.sp] = self.pc as u16; 
        self.sp = (self.sp + 1) & (STACK_SIZE - 1); 
        self.pc = nnn;
    }

    // Skip the next instruction if vx equals nn.
    fn execute_opcode_3xnn(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let nn = self.opcode & 0x00ff; 
        self.pc += if self.v[x as usize] == nn as u8 {4} else {2}; 
    }

    // Skip the next instruction if vx does not equal nn.
    fn execute_opcode_4xnn(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let nn = self.opcode & 0x00ff; 
        self.pc += if self.v[x as usize] != nn as u8 {4} else {2}; 
    }

    // Skip the next instruction if vx equals vy.
    fn execute_opcode_5xy0(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        self.pc += if self.v[x as usize] == self.v[y as usize] {4} else {2};
    }

    // Set vx to nn.
    fn execute_opcode_6xnn(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let nn = self.opcode & 0x00ff; 
        self.v[x as usize] = nn as u8; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Add nn to vx.
    fn execute_opcode_7xnn(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let nn = self.opcode & 0x00ff; 
        self.v[x as usize] = self.v[x as usize].wrapping_add(nn as u8);
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Set vx to the value of vy.
    fn execute_opcode_8xy0(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        self.v[x as usize] = self.v[y as usize]; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Set vx to vx OR vy.
    fn execute_opcode_8xy1(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        self.v[x as usize] |= self.v[y as usize]; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Set vx to vx AND vy.
    fn execute_opcode_8xy2(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        self.v[x as usize] &= self.v[y as usize]; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Set vx to vx XOR vy.
    fn execute_opcode_8xy3(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        self.v[x as usize] ^= self.v[y as usize]; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Add vy to vx and set vf to 1 if there was a carry, 0 otherwise. 
    fn execute_opcode_8xy4(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        let vx = self.v[x as usize]; 
        let vy = self.v[y as usize]; 
        self.v[x as usize] = vx.wrapping_add(vy); 
        let carried = (vx as u16 + vy as u16) > 0xff;
        self.v[0x0f] = if carried {1} else {0}; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Subtract vy from vx. Set vf to 0 if there was a borrow, 1 otherwise.
    fn execute_opcode_8xy5(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];
        self.v[x as usize] = vx.wrapping_sub(vy); 
        let borrowed = vy > vx;
        self.v[0x0f] = if borrowed {0} else {1}; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // There is some difference in opinion on how this opcode should
    // be implemented. See http://mattmik.com/emu.html
    //
    // This implementation mirrors the behavior of the original interpreter.
    //
    // Store the value of register vy shifted right one bit in register vx.
    // Set register vf to the least significant bit prior to the shift.
    #[allow(dead_code)]
    fn execute_opcode_8xy6_orig_not_used(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        self.v[0x0f] = self.v[y as usize] & 0x01;
        self.v[x as usize] = self.v[y as usize] >> 1; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // There is some difference in opinion on how this opcode should
    // be implemented. See http://mattmik.com/emu.html
    //
    // This implementation follows the most recent descriptions of the 
    // instruction set. This implementation (perhaps erroneous) were
    // what a majority of programmers had in mind. As a result, it seems
    // to work with a majority of roms. A significant number of the more
    // complex roms, e.g. Space Invaders, will ONLY work with this 
    // implementation.
    //
    // Shifts vx right by one. Set vf to the value of the least significant
    // bit of vx before the shift. 
    fn execute_opcode_8xy6(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        self.v[0x0f] = self.v[x as usize] & 0b00000001;
        self.v[x as usize] >>= 1;
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Set vx to vy minus vx. Set vf to 0 if there was a borrow, 1 otherwise.
    fn execute_opcode_8xy7(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        let vx = self.v[x as usize];
        let vy = self.v[y as usize];
        self.v[x as usize] = vy.wrapping_sub(vx); 
        let borrowed = vx > vy; 
        self.v[0x0f] = if borrowed {0} else {1}; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // There is some difference in opinion on how this opcode should
    // be implemented. See http://mattmik.com/emu.html
    //
    // This implementation mirrors the behavior of the original interpreter.
    // 
    // Store the value of register vy shifted left one bit in register vx.
    // Set register vf to the most significant bit prior to the shift.
    #[allow(dead_code)]
    fn execute_opcode_8xye_orig_not_used(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        self.v[0x0f] = (self.v[y as usize] >> 7) & 0b00000001;
        self.v[x as usize] = self.v[y as usize] << 1; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // There is some difference in opinion on how this opcode should
    // be implemented. See http://mattmik.com/emu.html
    //
    // This implementation follows the most recent descriptions of the 
    // instruction set. This implementation (perhaps erroneous) were
    // what a majority of programmers had in mind. As a result, it seems
    // to work with a majority of roms. A significant number of the more
    // complex roms, e.g. Space Invaders, will ONLY work with this 
    // implementation.
    //
    // Shift vx left by one. Set vf to the value of the most significant bit
    // of vx before the shift. Notice that vy is completely ignored. 
    fn execute_opcode_8xye(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        self.v[0x0f] = (self.v[x as usize] >> 7) & 0b00000001; 
        self.v[x as usize] <<= 1; 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Skip the next instruction if vx does not equal vy.
    fn execute_opcode_9xy0(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let y = (self.opcode & 0x00f0) >> 4; 
        self.pc += if self.v[x as usize] != self.v[y as usize] {4} else {2};
        self.pc &= 0x0fff;
    }

    // Set ram_idx to the address nnn.
    fn execute_opcode_annn(&mut self) {
        let nnn = self.opcode & 0x0fff; 
        self.ram_idx = nnn; 
        self.pc = (self.pc + 2) & 0x0fff; 
    } 

    // Jump to the address nnn plus v0.
    fn execute_opcode_bnnn(&mut self) {
        let nnn = self.opcode & 0x0fff; 
        self.pc = (nnn + (self.v[0] as u16)) & 0x0fff; 
    } 

    // Set vx to a random number and nn.
    fn execute_opcode_cxnn(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let nn = self.opcode & 0x00ff; 
        self.v[x as usize] = rand::random::<u8>() & (nn as u8); 
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Draw 8xn sprite from ram[ram_idx] at gfx[vx][vy]. Set vf to 1 if
    // any set pixels are changed to unset, and 0 otherwise. If n is 0 and
    // in SUPER mode, then show 16x16 sprite instead.
    fn execute_opcode_dxyn(&mut self) {
        let gfx_start_x = self.v[(self.opcode as usize & 0x0f00) >> 8] as usize;
        let gfx_start_y = self.v[(self.opcode as usize & 0x00f0) >> 4] as usize;
        let n = (self.opcode & 0x000f) as usize; 
        let sprt_w = if n == 0 && self.mode == Mode::SUPER {16} else {8};
        let sprt_h = if n == 0 {16} else {n};
        let sprt_bytes_per_row = sprt_w / 8; 
        self.v[0x0f] = 0x00;
        for y_offset in 0..sprt_h {
            for sprt_byte_col_idx in 0..sprt_bytes_per_row {
                let sprt_byte_ram_idx = self.ram_idx as usize + 
                    y_offset * sprt_bytes_per_row;
                let sprt_byte: u8 = self.ram[sprt_byte_ram_idx]; 
                for sprt_byte_bit_idx in 0..8 as usize {
                    let x_offset = sprt_byte_col_idx * 8 + sprt_byte_bit_idx;
                    // Drawing beyond max width and height will wrap.
                    let gfx_x = (gfx_start_x + x_offset) % self.width();
                    let gfx_y = (gfx_start_y + y_offset) % self.height(); 
                    // Mask to obtain single bit from byte. 
                    let mask = 0b_1000_0000_u8 >> sprt_byte_bit_idx; 
                    let sprt_pix = sprt_byte & mask != 0;
                    if sprt_pix == true {
                        let gfx_pix = &mut self.gfx[gfx_x][gfx_y];
                        *gfx_pix ^= true;
                        if *gfx_pix == true {
                            // Reduce flicker and draw only when pix switched on. 
                            self.draw = true;
                        } else {
                            self.v[0x0f] = 0x01;
                        } 
                    }
                }
            } 
        }
        self.pc = (self.pc + 2) & 0x0fff; 
    }
    
    // Skip the next instruction if the key stored in vx is pressed.
    fn execute_opcode_ex9e(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let key_pressed = self.keys[self.v[x as usize] as usize];
        self.pc = (self.pc + if key_pressed {4} else {2}) & 0x0fff;
    }

    // Skips the next instruction if the key stored in vx is not pressed.
    fn execute_opcode_exa1(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        let key_pressed = self.keys[self.v[x as usize] as usize];
        self.pc = (self.pc + if !key_pressed {4} else {2}) & 0x0fff;
    }

    // Set vx to the value of the delay timer.
    fn execute_opcode_fx07(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        self.v[x as usize] = self.dt;
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Wait for a keypress then store it in vx.
    // This implementation will only advance the program counter
    // if a keypress is found. In other words, this opcode will
    // execute over and over until a keypress is found. This allows
    // opportunity for a keypress to arrive in between executions.
    fn execute_opcode_fx0a(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8; 
        for i in 0..self.keys.len() {
            if self.keys[i] {
                self.v[x as usize] = i as u8;
                self.pc = (self.pc + 2) & 0x0fff; 
            }
        }
    }

    // Set the delay timer to vx.
    fn execute_opcode_fx15(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        self.dt = self.v[x as usize];
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Set the sound timer to vx.
    fn execute_opcode_fx18(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        self.st = self.v[x as usize];
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Add vx to ram_idx. Set vf to 1 if there was a range overflow,
    // ram_idx + vx > 0x0fff, 0 otherwise.
    fn execute_opcode_fx1e(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        let sum  = self.ram_idx + self.v[x as usize] as u16;
        let overflowed = sum > 0x0fff;
        self.v[0xf as usize] = if overflowed {1} else {0};
        self.ram_idx = sum % (0x0fff + 1);
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Set ram_idx to the location of the sprite for the character in vx. 
    // Characters 0-F are represented by a 4x5 font.
    fn execute_opcode_fx29(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        let fchar = self.v[x as usize];
        self.ram_idx = 0x0000 + (fchar as u16) * 5; 
        self.pc = (self.pc + 2) & 0x0fff; 
    } 

    // Set ram_idx to the location of the sprite for the character in vx, where
    // x must be in the range 0 to 9 (inclusive). Characters 0-F are 
    // represented by a 8x10 font.
    fn execute_opcode_fx30(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        let fchar = self.v[x as usize];
        self.ram_idx = 0x0000 + (fchar as u16) * 10; 
        self.pc = (self.pc + 2) & 0x0fff; 
    } 

    // Store the binary-coded decimal (BCD) representation of vx, with the
    // most significant of three digits at the address in ram_idx, the middle 
    // digit at ram_idx plus 1, and the least siginificant digit at ram_idx 
    // plus 2. In other words, take the decimal representation of vx, place 
    // the hundreds digit in memory at location in ram_idx, the tens digits 
    // at location ram_idx+1, and the ones digit at location ram_idx+2.
    fn execute_opcode_fx33(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        let mut vx = self.v[x as usize];
        let ones = vx % 10;
        vx /= 10;
        let tens = vx % 10;
        vx /= 10;
        let hundreds = vx % 10;
        self.ram[(self.ram_idx+0) as usize] = hundreds as u8;
        self.ram[(self.ram_idx+1) as usize] = tens as u8;
        self.ram[(self.ram_idx+2) as usize] = ones as u8;
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Store v0 to vx in memory starting at address ram_idx.
    fn execute_opcode_fx55(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        for i in 0..(x as u16) + 1 {
            self.ram[(self.ram_idx+i) as usize] = self.v[i as usize];
        }
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Fill v0 to vx with values from memory starting at address ram_idx.
    fn execute_opcode_fx65(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        for i in 0..(x as u16) + 1 {
            self.v[i as usize] = self.ram[(self.ram_idx+i) as usize];
        }
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Store v0 to vx in super_mode_rpl_flags user flags (x <= 7).
    fn execute_opcode_fx75(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        for i in 0..(cmp::min(x,7) as u16) + 1 {
            self.super_mode_rpl_flags[i as usize] = self.v[i as usize];
        }
        self.pc = (self.pc + 2) & 0x0fff; 
    }

    // Fill v0 to vx with values from super_mode_rpl_flags (x <= 7).
    fn execute_opcode_fx85(&mut self) {
        let x = (self.opcode & 0x0f00) >> 8;
        for i in 0..(cmp::min(x,7) as u16) + 1 {
            self.v[i as usize] = self.super_mode_rpl_flags[i as usize];
        }
        self.pc = (self.pc + 2) & 0x0fff; 
    }
    
    // Fetch the opcode to which the program counter is pointing.
    fn fetch_opcode(&mut self) {
        let hbyte = self.ram[self.pc as usize];
        let lbyte = self.ram[self.pc as usize + 1];
        // Uses big-endiannes for multi byte data types.
        self.opcode = (hbyte as u16) << 8 | lbyte as u16; 
    }
                
    fn decode_and_execute_opcode(&mut self) {
        match self.opcode & 0xf000 {
            0x0000 => 
                match self.opcode & 0x00f0 {
                    0x00c0 => self.execute_opcode_00cn(),
                    _ =>  match self.opcode & 0x00ff {
                        0x00e0 => self.execute_opcode_00e0(),
                        0x00ee => self.execute_opcode_00ee(),
                        0x00fb => self.execute_opcode_00fb(),
                        0x00fc => self.execute_opcode_00fc(),
                        0x00fd => self.execute_opcode_00fd(),
                        0x00fe => self.execute_opcode_00fe(),
                        0x00ff => self.execute_opcode_00ff(),
                        _ => self.unknown_opcode()
                }, 
            }, 
            0x1000 => self.execute_opcode_1nnn(), 
            0x2000 => self.execute_opcode_2nnn(), 
            0x3000 => self.execute_opcode_3xnn(), 
            0x4000 => self.execute_opcode_4xnn(), 
            0x5000 => match self.opcode & 0x000f {
                0x0000 => self.execute_opcode_5xy0(),   
                _ => self.unknown_opcode()
            }, 
            0x6000 => self.execute_opcode_6xnn(), 
            0x7000 => self.execute_opcode_7xnn(), 
            0x8000 => match self.opcode & 0x000f {
                0x0000 => self.execute_opcode_8xy0(),
                0x0001 => self.execute_opcode_8xy1(),
                0x0002 => self.execute_opcode_8xy2(),
                0x0003 => self.execute_opcode_8xy3(),
                0x0004 => self.execute_opcode_8xy4(),
                0x0005 => self.execute_opcode_8xy5(),
                0x0006 => self.execute_opcode_8xy6(),
                0x0007 => self.execute_opcode_8xy7(),
                0x000e => self.execute_opcode_8xye(),
                _ => self.unknown_opcode()
            }, 
            0x9000 => self.execute_opcode_9xy0(), 
            0xa000 => self.execute_opcode_annn(), 
            0xb000 => self.execute_opcode_bnnn(), 
            0xc000 => self.execute_opcode_cxnn(), 
            0xd000 => self.execute_opcode_dxyn(), 
            0xe000 => match self.opcode & 0x000f {
                0x000E => self.execute_opcode_ex9e(),
                0x0001 => self.execute_opcode_exa1(),
                _ => self.unknown_opcode()
            }, 
            0xf000 => match self.opcode & 0x00ff {
               0x0007 => self.execute_opcode_fx07(),
               0x000a => self.execute_opcode_fx0a(),
               0x0015 => self.execute_opcode_fx15(),
               0x0018 => self.execute_opcode_fx18(),
               0x001e => self.execute_opcode_fx1e(),
               0x0029 => self.execute_opcode_fx29(),
               0x0030 => self.execute_opcode_fx30(),
               0x0033 => self.execute_opcode_fx33(),
               0x0055 => self.execute_opcode_fx55(),
               0x0065 => self.execute_opcode_fx65(),
               0x0075 => self.execute_opcode_fx75(),
               0x0085 => self.execute_opcode_fx85(),
               _ => self.unknown_opcode()
            },
            _ => self.unknown_opcode()
        }
    }
    
    fn unknown_opcode(&self) -> ! {
        panic!("Unknown opcode: {:x}", self.opcode);    
    }

}

#[cfg(test)]
mod tests {

    use super::Emu;
    use super::{SMALL_GFX_H, SMALL_GFX_W};
    use super::super::{Mode, GFX_H, GFX_W};

    #[test]
    pub fn test_opcode_00cn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        emu.gfx[0][0] = true;
        emu.gfx[1][0] = true;
        //when
        emu.opcode = 0x00c2;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(false, emu.gfx[0][0]);
        assert_eq!(false, emu.gfx[1][0]);
        assert_eq!(true, emu.gfx[0][2]);
        assert_eq!(true, emu.gfx[1][2]);
        assert_eq!(true, emu.draw);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    pub fn test_opcode_00e0() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        emu.draw = false;
        for x in 0..GFX_W { for y in 0..GFX_H { emu.gfx[x][y] = true; } }
        //when
        emu.opcode = 0x00e0;
        emu.decode_and_execute_opcode();
        //then
        for x in 0..GFX_W { for y in 0..GFX_H { assert_eq!(false, emu.gfx[x][y]); } }
        assert_eq!(true, emu.draw);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    pub fn test_opcode_00fb() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        emu.gfx[0][0] = true;
        emu.gfx[1][0] = true;
        //when
        emu.opcode = 0x00fb;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(false, emu.gfx[0][0]);
        assert_eq!(false, emu.gfx[1][0]);
        assert_eq!(true, emu.gfx[4][0]);
        assert_eq!(true, emu.gfx[5][0]);
        assert_eq!(true, emu.draw);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    pub fn test_opcode_00fc() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        emu.gfx[4][0] = true;
        emu.gfx[5][0] = true;
        //when
        emu.opcode = 0x00fc;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(false, emu.gfx[4][0]);
        assert_eq!(false, emu.gfx[5][0]);
        assert_eq!(true, emu.gfx[0][0]);
        assert_eq!(true, emu.gfx[1][0]);
        assert_eq!(true, emu.draw);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    pub fn test_opcode_00ee() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0ccc; 
        emu.stack[0] = 0x0aaa;
        emu.stack[1] = 0x0bbb;
        emu.sp = 0x01;
        //when
        emu.opcode = 0x00ee;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x00, emu.sp);
        assert_eq!(0x0aaa+2, emu.pc);
    }

    #[test]
    pub fn test_opcode_00fe() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0aaa; 
        emu.mode = Mode::SUPER;
        //when
        emu.opcode = 0x00fe;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(Mode::STANDARD, emu.mode);
        assert_eq!(0x0aaa+2, emu.pc);
    }

    #[test]
    pub fn test_opcode_00ff() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0aaa; 
        emu.mode = Mode::STANDARD;
        //when
        emu.opcode = 0x00ff;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(Mode::SUPER, emu.mode);
        assert_eq!(0x0aaa+2, emu.pc);
    }

    #[test]
    pub fn test_opcode_1nnn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0aaa; 
        //when
        emu.opcode = 0x1bcd;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0bcd, emu.pc);
    }

    #[test]
    pub fn test_opcode_2nnn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        //when
        emu.opcode = 0x1234;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0234, emu.pc);
    }

    #[test]
    pub fn test_opcode_3xnn_given_vx_equals_nn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        //when
        emu.opcode = 0x3a23;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+4, emu.pc);
    }

    #[test]
    pub fn test_opcode_3xnn_given_vx_not_equals_nn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        //when
        emu.opcode = 0x3a24;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    pub fn test_opcode_4xnn_given_vx_equals_nn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        //when
        emu.opcode = 0x4a23;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    pub fn test_opcode_4xnn_given_vx_not_equals_nn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        //when
        emu.opcode = 0x4a24;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+4, emu.pc);
    }
    
    #[test]
    pub fn test_opcode_5xy0_given_vx_equals_vy() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        emu.v[0x0b] = 0x23;
        //when
        emu.opcode = 0x5ab0;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+4, emu.pc);
    }

    #[test]
    pub fn test_opcode_5xy0_given_vx_does_not_equal_vy() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        emu.v[0x0b] = 0x24;
        //when
        emu.opcode = 0x5ab0;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_6xnn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        //when
        emu.opcode = 0x6a24;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0024, emu.v[0x0a]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_7xnn_without_overflow() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x03;
        //when
        emu.opcode = 0x7afb;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0xfe, emu.v[0x0a]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_7xnn_with_overflow() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x03;
        //when
        emu.opcode = 0x7aff;
        emu.decode_and_execute_opcode();
        //then
        let wrap_mod = (0x0003u16 + 0x00ffu16) % (0x00ffu16 + 0x00001u16);
        assert_eq!(wrap_mod, (emu.v[0x0a] as u16));
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_8xy0() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        emu.v[0x0b] = 0x24;
        //when
        emu.opcode = 0x8ab0;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x24, emu.v[0x0a]);
        assert_eq!(0x24, emu.v[0x0b]);
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_8xy1() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        emu.v[0x0b] = 0x24;
        //when
        emu.opcode = 0x8ab1;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x23|0x24, emu.v[0x0a]);
        assert_eq!(0x24, emu.v[0x0b]);
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_8xy2() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        emu.v[0x0b] = 0x24;
        //when
        emu.opcode = 0x8ab2;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x23&0x24, emu.v[0x0a]);
        assert_eq!(0x24, emu.v[0x0b]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_8xy3() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x23;
        emu.v[0x0b] = 0x24;
        //when
        emu.opcode = 0x8ab3;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x23^0x24, emu.v[0x0a]);
        assert_eq!(0x24, emu.v[0x0b]);
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_8xy4_without_carry() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0xf0;
        emu.v[0x0b] = 0x03;
        //when
        emu.opcode = 0x8ab4;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0xf3, 0xf0 + 0x03);
        assert_eq!(0xf3, emu.v[0x0a]);
        assert_eq!(0x03, emu.v[0x0b]);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_8xy4_with_carry() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0xff;
        emu.v[0x0b] = 0x03;
        //when
        emu.opcode = 0x8ab4;
        emu.decode_and_execute_opcode();
        //then
        let wrap_mod = (0x00ffu16 + 0x0003u16) % (0x00ffu16 + 0x00001u16);
        assert_eq!(0x02u16, wrap_mod);
        assert_eq!(0x02, emu.v[0x0a]);
        assert_eq!(0x03, emu.v[0x0b]);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_8xy5_without_borrow() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x09;
        emu.v[0x0b] = 0x08;
        //when
        emu.opcode = 0x8ab5;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x01, 0x09 - 0x08);
        assert_eq!(0x01, emu.v[0x0a]);
        assert_eq!(0x08, emu.v[0x0b]);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_8xy5_with_borrow() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x08;
        emu.v[0x0b] = 0x09;
        //when
        emu.opcode = 0x8ab5;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0xff, emu.v[0x0a]);
        assert_eq!(0x09, emu.v[0x0b]);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_8xy6_orig_not_used_least_significant_bit_not_set() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x07;
        emu.v[0x0b] = 0x04;
        //when
        emu.opcode = 0x8ab6;
        emu.execute_opcode_8xy6_orig_not_used();
        //then
        assert_eq!(0x02, 0x04 >> 1);
        assert_eq!(0x02, emu.v[0x0a]);
        assert_eq!(0x04, emu.v[0x0b]);
        assert_eq!(0x00, emu.v[0x0b] & 0x01);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_8xy6_orig_not_used_least_significant_bit_set() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x04;
        emu.v[0x0b] = 0x05;
        //when
        emu.opcode = 0x8ab6;
        emu.execute_opcode_8xy6_orig_not_used();
        //then
        assert_eq!(0x02, 0x05 >> 1);
        assert_eq!(0x02, emu.v[0x0a]);
        assert_eq!(0x05, emu.v[0x0b]);
        assert_eq!(0x01, emu.v[0x0b] & 0x01);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_8xy6_least_significant_bit_not_set() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x04;
        emu.v[0x0b] = 0x07;
        //when
        emu.opcode = 0x8ab6;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x02, 0x04 >> 1);
        assert_eq!(0x02, emu.v[0x0a]);
        assert_eq!(0x07, emu.v[0x0b]);
        assert_eq!(0x00, emu.v[0x0a] & 0x01);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_8xy6_least_significant_bit_set() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x05;
        emu.v[0x0b] = 0x04;
        //when
        emu.opcode = 0x8ab6;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x02, 0x05 >> 1);
        assert_eq!(0x02, emu.v[0x0a]);
        assert_eq!(0x04, emu.v[0x0b]);
        assert_eq!(0x00, emu.v[0x0a] & 0x01);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_8xy7_without_borrow() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x08;
        emu.v[0x0b] = 0x09;
        //when
        emu.opcode = 0x8ab7;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x01, 0x09 - 0x08);
        assert_eq!(0x01, emu.v[0x0a]);
        assert_eq!(0x09, emu.v[0x0b]);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_8xy7_with_borrow() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x09;
        emu.v[0x0b] = 0x08;
        //when
        emu.opcode = 0x8ab7;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0xff, emu.v[0x0a]);
        assert_eq!(0x08, emu.v[0x0b]);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_8xye_most_significant_bit_not_set() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0b01111111_u8;
        emu.v[0x0b] = 0b11111111_u8;
        //when
        emu.opcode = 0x8abe;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0b11111110_u8, emu.v[0x0a]);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_8xye_most_significant_bit_set() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0b11111111_u8;
        emu.v[0x0b] = 0b01111111_u8;
        //when
        emu.opcode = 0x8abe;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0b11111110_u8, emu.v[0x0a]);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
   }

    #[test]
    fn test_opcode_8xye_orig_not_used_most_significant_bit_not_set() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0b11111111_u8;
        emu.v[0x0b] = 0b01111111_u8;
        //when
        emu.opcode = 0x8abe;
        emu.execute_opcode_8xye_orig_not_used();
        //then
        assert_eq!(0b11111110_u8, emu.v[0x0a]);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
   }

    #[test]
    fn test_opcode_8xye_orig_not_used_most_significant_bit_set() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0b01111111_u8;
        emu.v[0x0b] = 0b11111111_u8;
        //when
        emu.opcode = 0x8abe;
        emu.execute_opcode_8xye_orig_not_used();
        //then
        assert_eq!(0b11111110_u8, emu.v[0x0a]);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
   }

    #[test]
    fn test_opcode_9xy0_vx_does_not_match_vy() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x07;
        emu.v[0x0b] = 0x05;
        //when
        emu.opcode = 0x9ab0;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+4, emu.pc);
    }

    #[test]
    fn test_opcode_9xy0_vx_matches_vy() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x0a] = 0x07;
        emu.v[0x0b] = 0x07;
        //when
        emu.opcode = 0x9ab0;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_annn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.ram_idx = 0xacc;
        //when
        emu.opcode = 0xadef;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0def, emu.ram_idx);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_bnnn() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0] = 0x23;
        //when
        emu.opcode = 0xb345;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0368, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_simple_draw() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        emu.draw = false;
        emu.v[1] = 0x0005;
        emu.v[2] = 0x0006;
        emu.ram_idx = 0x222;
        emu.ram[(emu.ram_idx+0) as usize] = 0b01010101 as u8;
        emu.ram[(emu.ram_idx+1) as usize] = 0b11111111 as u8;

        //when
        emu.opcode = 0xd122;
        emu.decode_and_execute_opcode();

        //then
        assert_eq!(false, emu.gfx[0x0005+0][0x0006+0]);
        assert_eq!(true,  emu.gfx[0x0005+1][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+2][0x0006+0]);
        assert_eq!(true,  emu.gfx[0x0005+3][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+4][0x0006+0]);
        assert_eq!(true,  emu.gfx[0x0005+5][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+6][0x0006+0]);
        assert_eq!(true,  emu.gfx[0x0005+7][0x0006+0]);

        assert_eq!(true,  emu.gfx[0x0005+0][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+1][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+2][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+3][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+4][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+5][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+6][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+7][0x0006+1]);
        
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_simple_undraw() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        emu.draw = false;

        emu.gfx[0x0005+0][0x006+0] = false;
        emu.gfx[0x0005+1][0x006+0] = true;
        emu.gfx[0x0005+2][0x006+0] = false;
        emu.gfx[0x0005+3][0x006+0] = true;
        emu.gfx[0x0005+4][0x006+0] = false;
        emu.gfx[0x0005+5][0x006+0] = true;
        emu.gfx[0x0005+6][0x006+0] = false;
        emu.gfx[0x0005+7][0x006+0] = true;

        emu.gfx[0x0005+0][0x006+1] = true;
        emu.gfx[0x0005+1][0x006+1] = true;
        emu.gfx[0x0005+2][0x006+1] = true;
        emu.gfx[0x0005+3][0x006+1] = true;
        emu.gfx[0x0005+4][0x006+1] = true;
        emu.gfx[0x0005+5][0x006+1] = true;
        emu.gfx[0x0005+6][0x006+1] = true;
        emu.gfx[0x0005+7][0x006+1] = true;

        emu.v[1] = 0x0005;
        emu.v[2] = 0x0006;
        emu.ram_idx = 0x222;
        emu.ram[(emu.ram_idx+0) as usize] = 0b01010101 as u8;
        emu.ram[(emu.ram_idx+1) as usize] = 0b11111111 as u8;
        
        //when
        emu.opcode = 0xd122;
        emu.decode_and_execute_opcode();
        
        //then
        assert_eq!(false, emu.gfx[0x0005+0][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+1][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+2][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+3][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+4][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+5][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+6][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+7][0x0006+0]);

        assert_eq!(false, emu.gfx[0x0005+0][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+1][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+2][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+3][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+4][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+5][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+6][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+7][0x0006+1]);
        
        assert_eq!(false, emu.draw);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_simple_partial_redraw() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        emu.draw = false;

        emu.gfx[0x0005+0][0x006+0] = false;
        emu.gfx[0x0005+1][0x006+0] = true;
        emu.gfx[0x0005+2][0x006+0] = false;
        emu.gfx[0x0005+3][0x006+0] = true;
        emu.gfx[0x0005+4][0x006+0] = false;
        emu.gfx[0x0005+5][0x006+0] = false;
        emu.gfx[0x0005+6][0x006+0] = false;
        emu.gfx[0x0005+7][0x006+0] = false;

        emu.gfx[0x0005+0][0x006+1] = true;
        emu.gfx[0x0005+1][0x006+1] = true;
        emu.gfx[0x0005+2][0x006+1] = true;
        emu.gfx[0x0005+3][0x006+1] = true;
        emu.gfx[0x0005+4][0x006+1] = true;
        emu.gfx[0x0005+5][0x006+1] = true;
        emu.gfx[0x0005+6][0x006+1] = true;
        emu.gfx[0x0005+7][0x006+1] = true;

        emu.v[1] = 0x0005;
        emu.v[2] = 0x0006;
        emu.ram_idx = 0x222;
        emu.ram[(emu.ram_idx+0) as usize] = 0b11111111 as u8;
        emu.ram[(emu.ram_idx+1) as usize] = 0b11110000 as u8;
        
        //when
        emu.opcode = 0xd122;
        emu.decode_and_execute_opcode();
        
        //then
        assert_eq!(true,  emu.gfx[0x0005+0][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+1][0x0006+0]);
        assert_eq!(true,  emu.gfx[0x0005+2][0x0006+0]);
        assert_eq!(false, emu.gfx[0x0005+3][0x0006+0]);
        assert_eq!(true,  emu.gfx[0x0005+4][0x0006+0]);
        assert_eq!(true,  emu.gfx[0x0005+5][0x0006+0]);
        assert_eq!(true,  emu.gfx[0x0005+6][0x0006+0]);
        assert_eq!(true,  emu.gfx[0x0005+7][0x0006+0]);

        assert_eq!(false, emu.gfx[0x0005+0][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+1][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+2][0x0006+1]);
        assert_eq!(false, emu.gfx[0x0005+3][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+4][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+5][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+6][0x0006+1]);
        assert_eq!(true,  emu.gfx[0x0005+7][0x0006+1]);
        
        assert_eq!(true, emu.draw);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_overflow_width() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        emu.draw = false;
        emu.v[1] = (SMALL_GFX_W - 4) as u8;
        emu.v[2] = 0x0006 ;
        emu.ram_idx = 0x222;
        emu.ram[(emu.ram_idx+0) as usize] = 0b01010101 as u8;
        emu.ram[(emu.ram_idx+1) as usize] = 0b11111111 as u8;

        //when
        emu.opcode = 0xd122;
        emu.decode_and_execute_opcode();

        //then
        assert_eq!(false, emu.gfx[SMALL_GFX_W-4+0][0x0006+0]);
        assert_eq!(true,  emu.gfx[SMALL_GFX_W-4+1][0x0006+0]);
        assert_eq!(false, emu.gfx[SMALL_GFX_W-4+2][0x0006+0]);
        assert_eq!(true,  emu.gfx[SMALL_GFX_W-4+3][0x0006+0]);
        assert_eq!(false, emu.gfx[0][0x0006+0]);
        assert_eq!(true,  emu.gfx[1][0x0006+0]);
        assert_eq!(false, emu.gfx[2][0x0006+0]);
        assert_eq!(true,  emu.gfx[3][0x0006+0]);

        assert_eq!(true, emu.gfx[SMALL_GFX_W-4+0][0x0006+1]);
        assert_eq!(true, emu.gfx[SMALL_GFX_W-4+1][0x0006+1]);
        assert_eq!(true, emu.gfx[SMALL_GFX_W-4+2][0x0006+1]);
        assert_eq!(true, emu.gfx[SMALL_GFX_W-4+3][0x0006+1]);
        assert_eq!(true, emu.gfx[0][0x0006+1]);
        assert_eq!(true, emu.gfx[1][0x0006+1]);
        assert_eq!(true, emu.gfx[2][0x0006+1]);
        assert_eq!(true, emu.gfx[3][0x0006+1]);
        
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_overflow_height() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000; 
        emu.draw = false;
        emu.v[1] = 0x0005;
        emu.v[2] = (GFX_H - 1) as u8;
        emu.ram_idx = 0x222;
        emu.ram[(emu.ram_idx+0) as usize] = 0b01010101 as u8;
        emu.ram[(emu.ram_idx+1) as usize] = 0b11111111 as u8;

        //when
        emu.opcode = 0xd122;
        emu.decode_and_execute_opcode();

        //then
        assert_eq!(false, emu.gfx[0x0005+0][SMALL_GFX_H-1]);
        assert_eq!(true,  emu.gfx[0x0005+1][SMALL_GFX_H-1]);
        assert_eq!(false, emu.gfx[0x0005+2][SMALL_GFX_H-1]);
        assert_eq!(true,  emu.gfx[0x0005+3][SMALL_GFX_H-1]);
        assert_eq!(false, emu.gfx[0x0005+4][SMALL_GFX_H-1]);
        assert_eq!(true,  emu.gfx[0x0005+5][SMALL_GFX_H-1]);
        assert_eq!(false, emu.gfx[0x0005+6][SMALL_GFX_H-1]);
        assert_eq!(true,  emu.gfx[0x0005+7][SMALL_GFX_H-1]);

        assert_eq!(true,  emu.gfx[0x0005+0][0]);
        assert_eq!(true,  emu.gfx[0x0005+1][0]);
        assert_eq!(true,  emu.gfx[0x0005+2][0]);
        assert_eq!(true,  emu.gfx[0x0005+3][0]);
        assert_eq!(true,  emu.gfx[0x0005+4][0]);
        assert_eq!(true,  emu.gfx[0x0005+5][0]);
        assert_eq!(true,  emu.gfx[0x0005+6][0]);
        assert_eq!(true,  emu.gfx[0x0005+7][0]);
        
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_dxyn_draw_font_0() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x0; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_1() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x1; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("  # "), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte(" ## "), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("  # "), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("  # "), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte(" ###"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_2() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x2; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("   #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_3() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x3; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("   #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("   #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_4() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x4; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("   #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("   #"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_5() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x5; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("   #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_6() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x6; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_7() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x7; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("   #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("  # "), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte(" #  "), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte(" #  "), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_8() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x8; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_9() {
        let mut emu = Emu::new();
        //given
        let fchar = 0x9; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("   #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_A() {
        let mut emu = Emu::new();
        //given
        let fchar = 0xA; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_B() {
        let mut emu = Emu::new();
        //given
        let fchar = 0xB; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("### "), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("### "), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("### "), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_dxyn_draw_font_C() {
        let mut emu = Emu::new();
        //given
        let fchar = 0xC; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_D() {
        let mut emu = Emu::new();
        //given
        let fchar = 0xD; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("### "), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#  #"), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("### "), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_dxyn_draw_font_E() {
        let mut emu = Emu::new();
        //given
        let fchar = 0xE; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }
    
    #[test]
    fn test_opcode_dxyn_draw_font_F() {
        let mut emu = Emu::new();
        //given
        let fchar = 0xF; 
        emu.ram_idx = 0x0000 + (fchar as u16) * 5; 
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xd005;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 0));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 1));
        assert_eq!(txt_to_byte("####"), booleans_to_byte(&emu.gfx, 0, 2));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 3));
        assert_eq!(txt_to_byte("#   "), booleans_to_byte(&emu.gfx, 0, 4));
        assert_eq!(true, emu.draw);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    fn txt_to_byte(txt: &str) -> u8 {
        let mut bits: u8 = 0b000000000;
        for (i,c) in txt.chars().enumerate() {
            bits |= if c == '#' {0b10000000} else {0b00000000} >> i;
        }
        bits
    }

    fn booleans_to_byte(gfx: &[[bool; GFX_H]; GFX_W], 
                        x: usize, y: usize) -> u8 {
        let mut bits: u8 = 0b00000000;
        for i in 0..8 {
            bits |= if gfx[x+i][y] {0b10000000} else {0b00000000} >> i; 
        }
        bits
    }

    #[test]
    fn test_opcode_ex9e_key_not_pressed() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[2] = 0x0a;
        emu.keys[0x0a] = false;
        //when
        emu.opcode = 0xe29e;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_ex9e_key_pressed() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[2] = 0x0a;
        emu.keys[0x0a] = true;
        //when
        emu.opcode = 0xe29e;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+4, emu.pc);
    }

    #[test]
    fn test_opcode_exa1_key_not_pressed() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[2] = 0x0a;
        emu.keys[0x0a] = false;
        //when
        emu.opcode = 0xe2a1;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+4, emu.pc);
    }

    #[test]
    fn test_opcode_exa1_key_pressed() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[2] = 0x0a;
        emu.keys[0x0a] = true;
        //when
        emu.opcode = 0xe2a1;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx07() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.dt = 0x9a;
        //when
        emu.opcode = 0xf207;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x9a, emu.v[0x02]);
        assert_eq!(0x9a, emu.dt);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx0a_with_keypress() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.keys[0x0f] = true;
        //when
        emu.opcode = 0xf20a;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0f, emu.v[0x02]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx0a_without_keypress() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        //when
        emu.opcode = 0xf20a;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+0, emu.pc);
    }

    #[test]
    fn test_opcode_fx15() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x02] = 0x9a;
        //when
        emu.opcode = 0xf215;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x9a, emu.v[0x02]);
        assert_eq!(0x9a, emu.dt);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx18() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0x02] = 0x9a;
        //when
        emu.opcode = 0xf218;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x9a, emu.v[0x02]);
        assert_eq!(0x9a, emu.st);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx1e_without_overflow() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.ram_idx = 0x222;
        emu.v[0x02] = 0xab;
        //when
        emu.opcode = 0xf21e;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x2cd, 0x222 + 0xab);
        assert_eq!(0x2cd, emu.ram_idx);
        assert_eq!(0xab, emu.v[0x02]);
        assert_eq!(0x00, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx1e_with_overflow() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.ram_idx = 0xfff;
        emu.v[0x02] = 0xab;
        //when
        emu.opcode = 0xf21e;
        emu.decode_and_execute_opcode();
        //then
        let wrap_mod = (0xfff + 0xab) % (0xfff + 0x001);
        assert_eq!(0x0aa, wrap_mod);
        assert_eq!(0x0aa, emu.ram_idx);
        assert_eq!(0xab, emu.v[0x02]);
        assert_eq!(0x01, emu.v[0x0f]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx29() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.ram_idx = 0xfff;
        emu.v[0x03] = 0x0a;
        //when
        emu.opcode = 0xf329;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+(0x0a*5), emu.ram_idx);
        assert_eq!(0x0a, emu.v[0x03]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx30() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.ram_idx = 0xfff;
        emu.v[0x03] = 0x0a;
        //when
        emu.opcode = 0xf330;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0000+(0x0a*10), emu.ram_idx);
        assert_eq!(0x0a, emu.v[0x03]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx33() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.ram_idx = 0xbbb;
        emu.v[0x02] = 0x7b;
        //when
        emu.opcode = 0xf233;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x7b, 123);
        assert_eq!(0x7b, emu.v[0x02]);
        assert_eq!(1, emu.ram[(emu.ram_idx+0) as usize]);
        assert_eq!(2, emu.ram[(emu.ram_idx+1) as usize]);
        assert_eq!(3, emu.ram[(emu.ram_idx+2) as usize]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx55() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.ram_idx = 0x333;
        emu.v[0x00] = 0x0a;
        emu.v[0x01] = 0x0b;
        emu.v[0x02] = 0x0c;
        //when
        emu.opcode = 0xf355;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0a, emu.ram[(emu.ram_idx+0) as usize]);
        assert_eq!(0x0b, emu.ram[(emu.ram_idx+1) as usize]);
        assert_eq!(0x0c, emu.ram[(emu.ram_idx+2) as usize]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx65() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.ram_idx = 0x333;
        emu.ram[(emu.ram_idx + 0) as usize] = 0x0a;
        emu.ram[(emu.ram_idx + 1) as usize] = 0x0b;
        emu.ram[(emu.ram_idx + 2) as usize] = 0x0c;
        //when
        emu.opcode = 0xf365;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x0a, emu.v[0]);
        assert_eq!(0x0b, emu.v[1]);
        assert_eq!(0x0c, emu.v[2]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx75() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0] = 0x03;
        emu.v[1] = 0x04;
        emu.v[2] = 0x05;
        emu.v[3] = 0x06;
        emu.v[4] = 0x07;
        emu.v[5] = 0x08;
        emu.v[6] = 0x09;
        emu.v[7] = 0x0A;
        emu.v[8] = 0x0B;
        //when
        emu.opcode = 0xf375;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x03, emu.super_mode_rpl_flags[0]);
        assert_eq!(0x04, emu.super_mode_rpl_flags[1]);
        assert_eq!(0x05, emu.super_mode_rpl_flags[2]);
        assert_eq!(0x06, emu.super_mode_rpl_flags[3]);
        assert_eq!(0x00, emu.super_mode_rpl_flags[4]);
        assert_eq!(0x00, emu.super_mode_rpl_flags[5]);
        assert_eq!(0x00, emu.super_mode_rpl_flags[6]);
        assert_eq!(0x00, emu.super_mode_rpl_flags[7]);
        assert_eq!(0x0000+2, emu.pc);
    }

    #[test]
    fn test_opcode_fx75_safe_against_x_greater_than_7() {
        let mut emu = Emu::new();
        //given
        emu.pc = 0x0000;
        emu.v[0] = 0x03;
        emu.v[1] = 0x04;
        emu.v[2] = 0x05;
        emu.v[3] = 0x06;
        emu.v[4] = 0x07;
        emu.v[5] = 0x08;
        emu.v[6] = 0x09;
        emu.v[7] = 0x0A;
        emu.v[8] = 0x0B;
        //when
        emu.opcode = 0xf875;
        emu.decode_and_execute_opcode();
        //then
        assert_eq!(0x03, emu.super_mode_rpl_flags[0]);
        assert_eq!(0x04, emu.super_mode_rpl_flags[1]);
        assert_eq!(0x05, emu.super_mode_rpl_flags[2]);
        assert_eq!(0x06, emu.super_mode_rpl_flags[3]);
        assert_eq!(0x07, emu.super_mode_rpl_flags[4]);
        assert_eq!(0x08, emu.super_mode_rpl_flags[5]);
        assert_eq!(0x09, emu.super_mode_rpl_flags[6]);
        assert_eq!(0x0A, emu.super_mode_rpl_flags[7]);
        assert_eq!(0x0000+2, emu.pc);
    }

}

