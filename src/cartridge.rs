use crate::types::*;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    pub mbc_type: MbcType,
    pub rom_bank: u16,
    pub ram_bank: u8,
    pub ram_enable: bool,
    pub ram_enable_inv: bool,
    pub num_rom_banks: i32,
    pub num_ram_banks: i32,
    pub mbc1_mode: u8,
    pub mbc1_shift: u8,
    pub rom_size: u8,
    pub dirty: bool,
}

impl Cartridge {
    pub fn new() -> Self {
        Self {
            rom: vec![0u8; 0x8000],
            ram: vec![0u8; 0x8000],
            mbc_type: MbcType::None,
            rom_bank: 1,
            ram_bank: 0,
            ram_enable: false,
            ram_enable_inv: false,
            num_rom_banks: 2,
            num_ram_banks: 0,
            mbc1_mode: 0,
            mbc1_shift: 0,
            rom_size: 0,
            dirty: false,
        }
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> bool {
        let mut f = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return false,
        };
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_err() {
            return false;
        }
        if buf.len() < 0x8000 {
            buf.resize(0x8000, 0);
        }
        self.rom = buf;

        let cart_type = self.rom[0x147];
        self.rom_size = self.rom[0x148];
        let ram_size = self.rom[0x149];

        self.num_rom_banks = if self.rom_size <= 8 {
            2 << self.rom_size as i32
        } else {
            128
        };

        self.mbc_type = match cart_type {
            0x00 => MbcType::None,
            0x01 | 0x02 | 0x03 => MbcType::Mbc1,
            0x05 | 0x06 => MbcType::Mbc2,
            0x0F | 0x10 | 0x11 | 0x12 | 0x13 => MbcType::Mbc3,
            0x19 | 0x1A | 0x1B | 0x1C | 0x1D | 0x1E => MbcType::Mbc5,
            _ => MbcType::None,
        };

        self.num_ram_banks = match ram_size {
            0 => 0,
            1 => 1,
            2 => 1,
            3 => 4,
            4 => 16,
            5 => 8,
            _ => 0,
        };

        if self.num_ram_banks > 0 {
            self.ram = vec![0u8; (self.num_ram_banks as usize) * 0x2000];
        } else {
            self.ram = vec![0u8; 0x2000];
        }

        true
    }

    pub fn save_ram<P: AsRef<Path>>(&self, path: P) -> bool {
        if self.ram.is_empty() {
            return false;
        }
        let mut f = match File::create(&path) {
            Ok(f) => f,
            Err(_) => return false,
        };
        let sz = self.ram_size();
        f.write_all(&self.ram[..sz]).is_ok()
    }

    pub fn load_ram<P: AsRef<Path>>(&mut self, path: P) -> bool {
        if self.ram.is_empty() {
            return false;
        }
        let mut f = match File::open(&path) {
            Ok(f) => f,
            Err(_) => return false,
        };
        let sz = self.ram_size();
        let mut buf = vec![0u8; sz];
        if f.read(&mut buf).is_err() {
            return false;
        }
        for (i, b) in buf.into_iter().enumerate() {
            if i < self.ram.len() {
                self.ram[i] = b;
            }
        }
        true
    }

    pub fn ram_size(&self) -> usize {
        if self.num_ram_banks > 0 {
            (self.num_ram_banks as usize) * 0x2000
        } else {
            0x2000
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr < 0x4000 {
            return self.rom[addr as usize];
        }
        if addr < 0x8000 {
            let mut bank = self.rom_bank as u32;
            if self.mbc_type == MbcType::Mbc1 && self.mbc1_mode == 0 && self.rom_size >= 5 {
                bank &= !0x20;
            }
            let mut real_bank = bank % (self.num_rom_banks as u32);
            if real_bank == 0 {
                real_bank = 1;
            }
            let mut offset = (addr as u32 - 0x4000) + real_bank * 0x4000;
            if offset >= self.rom.len() as u32 {
                offset %= self.rom.len() as u32;
            }
            return self.rom[offset as usize];
        }
        if addr >= 0xA000 && addr < 0xC000 {
            if !self.ram_enable {
                return 0xFF;
            }
            if self.mbc_type == MbcType::Mbc1 && self.mbc1_mode == 1 {
                let mut bank = self.ram_bank as u32;
                if self.num_ram_banks > 0 {
                    bank %= self.num_ram_banks as u32;
                }
                if bank == 0 && self.num_ram_banks <= 1 {
                    let offset = addr as u32 - 0xA000;
                    return if (offset as usize) < self.ram.len() {
                        self.ram[offset as usize]
                    } else {
                        0xFF
                    };
                }
                let offset = (addr as u32 - 0xA000) + bank * 0x2000;
                return if (offset as usize) < self.ram.len() {
                    self.ram[offset as usize]
                } else {
                    0xFF
                };
            }
            let offset = addr as u32 - 0xA000;
            return if (offset as usize) < self.ram.len() {
                self.ram[offset as usize]
            } else {
                0xFF
            };
        }
        0xFF
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if addr < 0x2000 {
            match self.mbc_type {
                MbcType::Mbc1 | MbcType::Mbc3 | MbcType::Mbc5 => {
                    self.ram_enable = (val & 0x0F) == 0x0A;
                }
                MbcType::Mbc2 => {
                    if (addr & 0x0100) == 0 {
                        self.ram_enable = (val & 0x0F) == 0x0A;
                    }
                }
                _ => {}
            }
        } else if addr < 0x4000 {
            match self.mbc_type {
                MbcType::Mbc1 => {
                    let mut bank = val & 0x1F;
                    if bank == 0 {
                        bank = 1;
                    }
                    self.rom_bank = (self.rom_bank & 0x60) | (bank as u16);
                }
                MbcType::Mbc3 => {
                    let mut bank = val & 0x7F;
                    if bank == 0 {
                        bank = 1;
                    }
                    self.rom_bank = bank as u16;
                }
                MbcType::Mbc5 => {
                    if addr < 0x3000 {
                        self.rom_bank = (self.rom_bank & 0x100) | (val as u16);
                    } else {
                        self.rom_bank = (self.rom_bank & 0xFF) | (((val & 1) as u16) << 8);
                    }
                }
                MbcType::Mbc2 => {
                    if (addr & 0x0100) != 0 {
                        let mut bank = val & 0x0F;
                        if bank == 0 {
                            bank = 1;
                        }
                        self.rom_bank = bank as u16;
                    }
                }
                _ => {}
            }
        } else if addr < 0x6000 {
            match self.mbc_type {
                MbcType::Mbc1 => {
                    if self.mbc1_mode == 0 {
                        let bank = val & 3;
                        self.rom_bank = (self.rom_bank & 0x1F) | ((bank as u16) << 5);
                    } else {
                        self.ram_bank = val & 3;
                    }
                }
                MbcType::Mbc3 => {
                    self.ram_bank = val & 0x03;
                }
                MbcType::Mbc5 => {
                    self.ram_bank = val & 0x0F;
                }
                _ => {}
            }
        } else if addr < 0x8000 {
            if self.mbc_type == MbcType::Mbc1 {
                self.mbc1_mode = val & 1;
            }
        } else if addr >= 0xA000 && addr < 0xC000 {
            if !self.ram_enable {
                return;
            }
            let mut offset = (addr as u32) - 0xA000;
            if self.mbc_type == MbcType::Mbc1 && self.mbc1_mode == 1 && self.num_ram_banks > 1 {
                offset += (self.ram_bank as u32) * 0x2000;
            }
            if (offset as usize) < self.ram.len() {
                self.ram[offset as usize] = val;
                self.dirty = true;
            }
        }
    }
}
