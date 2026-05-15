use crate::types::*;
use crate::cartridge::Cartridge;
use crate::input::Input;
use crate::apu::Apu;

pub struct Memory {
    pub cart: Cartridge,
    pub input: Input,
    pub apu: Apu,
    pub vram: [u8; 0x2000],
    pub wram: [u8; 0x2000],
    pub oam: [u8; 0xA0],
    pub hram: [u8; 0x80],
    pub io_regs: [u8; 0x80],
    pub ie_register: u8,
    pub boot_rom: [u8; 0x100],
    pub boot_rom_active: bool,
}

impl Memory {
    pub fn new() -> Self {
        let mut m = Self {
            cart: Cartridge::new(),
            input: Input::new(),
            apu: Apu::new(),
            vram: [0u8; 0x2000],
            wram: [0u8; 0x2000],
            oam: [0u8; 0xA0],
            hram: [0u8; 0x80],
            io_regs: [0u8; 0x80],
            ie_register: 0,
            boot_rom: [0u8; 0x100],
            boot_rom_active: true,
        };
        m.reset();
        m
    }

    pub fn load_boot_rom(&mut self, data: &[u8]) {
        let n = data.len().min(self.boot_rom.len());
        self.boot_rom[..n].copy_from_slice(&data[..n]);
        self.boot_rom_active = true;
    }

    pub fn reset(&mut self) {
        self.vram.fill(0);
        self.wram.fill(0);
        self.oam.fill(0);
        self.hram.fill(0);
        self.io_regs.fill(0);
        self.ie_register = 0;

        self.io_regs[(NR10 as usize) - 0xFF00] = 0x80;
        self.io_regs[(NR11 as usize) - 0xFF00] = 0xBF;
        self.io_regs[(NR12 as usize) - 0xFF00] = 0xF3;
        self.io_regs[(NR14 as usize) - 0xFF00] = 0xBF;
        self.io_regs[(NR21 as usize) - 0xFF00] = 0x3F;
        self.io_regs[(NR24 as usize) - 0xFF00] = 0xBF;
        self.io_regs[(NR30 as usize) - 0xFF00] = 0x7F;
        self.io_regs[(NR31 as usize) - 0xFF00] = 0xFF;
        self.io_regs[(NR32 as usize) - 0xFF00] = 0x9F;
        self.io_regs[(NR34 as usize) - 0xFF00] = 0xBF;
        self.io_regs[(NR41 as usize) - 0xFF00] = 0xFF;
        self.io_regs[(NR44 as usize) - 0xFF00] = 0xBF;
        self.io_regs[(NR50 as usize) - 0xFF00] = 0x77;
        self.io_regs[(NR51 as usize) - 0xFF00] = 0xF3;
        self.io_regs[(NR52 as usize) - 0xFF00] = 0xF1;
    }

    pub fn read(&self, addr: u16) -> u8 {
        if self.boot_rom_active && addr < 0x0100 {
            return self.boot_rom[addr as usize];
        }
        if addr < 0x8000 {
            return self.cart.read(addr);
        }
        if addr < 0xA000 { return self.vram[(addr & 0x1FFF) as usize]; }
        if addr < 0xC000 {
            return self.cart.read(addr);
        }
        if addr < 0xE000 { return self.wram[(addr & 0x1FFF) as usize]; }
        if addr < 0xFE00 { return self.wram[(addr & 0x1FFF) as usize]; }
        if addr < 0xFEA0 { return self.oam[(addr & 0xFF) as usize]; }
        if addr == 0xFF00 {
            return self.input.read_p1();
        }
        if (0xFF04..=0xFF07).contains(&addr) { return self.io_regs[(addr - 0xFF00) as usize]; }
        if addr == 0xFF0F { return self.io_regs[0x0F]; }
        if (0xFF10..=0xFF3F).contains(&addr) { return self.io_regs[(addr - 0xFF00) as usize]; }
        if (0xFF40..=0xFF4B).contains(&addr) { return self.io_regs[(addr - 0xFF00) as usize]; }
        if (0xFF4F..=0xFF53).contains(&addr) { return self.io_regs[(addr - 0xFF00) as usize]; }
        if (0xFF68..=0xFF6F).contains(&addr) { return self.io_regs[(addr - 0xFF00) as usize]; }
        if (0xFF70..=0xFF7F).contains(&addr) { return self.io_regs[(addr - 0xFF00) as usize]; }
        if (0xFF80..0xFFFF).contains(&addr) { return self.hram[(addr & 0x7F) as usize]; }
        if addr == 0xFFFF { return self.ie_register; }
        0xFF
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        if addr < 0x8000 {
            self.cart.write(addr, val);
        } else if addr < 0xA000 {
            self.vram[(addr & 0x1FFF) as usize] = val;
        } else if addr < 0xC000 {
            self.cart.write(addr, val);
        } else if addr < 0xE000 {
            self.wram[(addr & 0x1FFF) as usize] = val;
        } else if addr < 0xFE00 {
            self.wram[(addr & 0x1FFF) as usize] = val;
        } else if addr < 0xFEA0 {
            self.oam[(addr & 0xFF) as usize] = val;
        } else if (0xFF80..0xFFFF).contains(&addr) {
            self.hram[(addr & 0x7F) as usize] = val;
        } else if addr == 0xFFFF {
            self.ie_register = val;
        } else if addr == 0xFF00 {
            self.input.write_p1(val);
        } else if addr == 0xFF46 {
            let src = (val as u16) << 8;
            for i in 0..0xA0u16 {
                let v = self.read(src + i);
                self.write(0xFE00 + i, v);
            }
        } else if addr == 0xFF50 {
            self.boot_rom_active = false;
        } else if addr == DIV {
            self.io_regs[(addr - 0xFF00) as usize] = 0;
        } else if (0xFF10..=0xFF3F).contains(&addr) {
            self.io_regs[(addr - 0xFF00) as usize] = val;
            self.apu.write_reg(addr, val);
        } else if (0xFF00..0xFF80).contains(&addr) {
            self.io_regs[(addr - 0xFF00) as usize] = val;
        }
    }

    pub fn read16(&self, addr: u16) -> u16 {
        (self.read(addr) as u16) | ((self.read(addr.wrapping_add(1)) as u16) << 8)
    }

    pub fn write16(&mut self, addr: u16, val: u16) {
        self.write(addr, (val & 0xFF) as u8);
        self.write(addr.wrapping_add(1), ((val >> 8) & 0xFF) as u8);
    }
}
