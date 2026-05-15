#![allow(dead_code)]

pub type U8  = u8;
pub type U16 = u16;
pub type U32 = u32;
pub type U64 = u64;
pub type S8  = i8;
pub type S16 = i16;

pub const LCDC: u16 = 0xFF40;
pub const STAT: u16 = 0xFF41;
pub const SCY:  u16 = 0xFF42;
pub const SCX:  u16 = 0xFF43;
pub const LY:   u16 = 0xFF44;
pub const LYC:  u16 = 0xFF45;
pub const DMA:  u16 = 0xFF46;
pub const BGP:  u16 = 0xFF47;
pub const OBP0: u16 = 0xFF48;
pub const OBP1: u16 = 0xFF49;
pub const WY:   u16 = 0xFF4A;
pub const WX:   u16 = 0xFF4B;
pub const P1:   u16 = 0xFF00;
pub const SB:   u16 = 0xFF01;
pub const SC:   u16 = 0xFF02;
pub const DIV:  u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05;
pub const TMA:  u16 = 0xFF06;
pub const TAC:  u16 = 0xFF07;
pub const IF:   u16 = 0xFF0F;
pub const IE:   u16 = 0xFFFF;
pub const NR10: u16 = 0xFF10;
pub const NR11: u16 = 0xFF11;
pub const NR12: u16 = 0xFF12;
pub const NR13: u16 = 0xFF13;
pub const NR14: u16 = 0xFF14;
pub const NR21: u16 = 0xFF16;
pub const NR22: u16 = 0xFF17;
pub const NR23: u16 = 0xFF18;
pub const NR24: u16 = 0xFF19;
pub const NR30: u16 = 0xFF1A;
pub const NR31: u16 = 0xFF1B;
pub const NR32: u16 = 0xFF1C;
pub const NR33: u16 = 0xFF1D;
pub const NR34: u16 = 0xFF1E;
pub const NR41: u16 = 0xFF20;
pub const NR42: u16 = 0xFF21;
pub const NR43: u16 = 0xFF22;
pub const NR44: u16 = 0xFF23;
pub const NR50: u16 = 0xFF24;
pub const NR51: u16 = 0xFF25;
pub const NR52: u16 = 0xFF26;
pub const BOOT: u16 = 0xFF50;

pub const INT_VBLANK: u8 = 1;
pub const INT_STAT:   u8 = 2;
pub const INT_TIMER:  u8 = 4;
pub const INT_SERIAL: u8 = 8;
pub const INT_JOYPAD: u8 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PpuMode {
    Hblank  = 0,
    Vblank  = 1,
    OamScan = 2,
    Draw    = 3,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MbcType {
    None,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
}

pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;
pub const CYCLES_PER_FRAME: u32 = 70224;
