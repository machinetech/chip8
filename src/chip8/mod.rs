
#[derive(Copy,Clone)]
pub enum Mode { STANDARD, SUPER }

pub const GFX_W: usize = 128;
pub const GFX_H: usize = 64;

pub mod emu;
pub mod metro;
pub mod ui;
pub mod wav;
