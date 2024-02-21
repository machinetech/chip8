pub const GFX_W: usize = 132;
pub const GFX_H: usize = 64;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Mode { Standard, Super }

pub mod emu;
pub mod metro;
pub mod ui;
pub mod wav;
