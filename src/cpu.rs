#![allow(unused_assignments)]

use crate::memory::Memory;
use crate::types::*;

pub struct Cpu {
    pub a: u8, pub f: u8,
    pub b: u8, pub c: u8,
    pub d: u8, pub e: u8,
    pub h: u8, pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub halted: bool,
    pub stop_mode: bool,
    pub ime: bool,
    pub ime_scheduled: u8,
}

impl Cpu {
    pub fn new() -> Self {
        let mut c = Self {
            a: 0, f: 0, b: 0, c: 0, d: 0, e: 0, h: 0, l: 0,
            sp: 0, pc: 0,
            halted: false,
            stop_mode: false,
            ime: false,
            ime_scheduled: 0,
        };
        c.reset();
        c
    }

    pub fn reset(&mut self) {
        self.set_af(0x01B0);
        self.set_bc(0x0013);
        self.set_de(0x00D8);
        self.set_hl(0x014D);
        self.sp = 0xFFFE;
        self.pc = 0x0000;
        self.halted = false;
        self.stop_mode = false;
        self.ime = false;
        self.ime_scheduled = 0;
    }

    #[inline] pub fn af(&self) -> u16 { ((self.a as u16) << 8) | self.f as u16 }
    #[inline] pub fn bc(&self) -> u16 { ((self.b as u16) << 8) | self.c as u16 }
    #[inline] pub fn de(&self) -> u16 { ((self.d as u16) << 8) | self.e as u16 }
    #[inline] pub fn hl(&self) -> u16 { ((self.h as u16) << 8) | self.l as u16 }

    #[inline] pub fn set_af(&mut self, v: u16) { self.a = (v >> 8) as u8; self.f = (v & 0xF0) as u8; }
    #[inline] pub fn set_bc(&mut self, v: u16) { self.b = (v >> 8) as u8; self.c = v as u8; }
    #[inline] pub fn set_de(&mut self, v: u16) { self.d = (v >> 8) as u8; self.e = v as u8; }
    #[inline] pub fn set_hl(&mut self, v: u16) { self.h = (v >> 8) as u8; self.l = v as u8; }

    #[inline] pub fn set_z(&mut self, v: bool) { self.f = (self.f & !0x80) | if v { 0x80 } else { 0 }; }
    #[inline] pub fn set_n(&mut self, v: bool) { self.f = (self.f & !0x40) | if v { 0x40 } else { 0 }; }
    #[inline] pub fn set_h(&mut self, v: bool) { self.f = (self.f & !0x20) | if v { 0x20 } else { 0 }; }
    #[inline] pub fn set_cf(&mut self, v: bool) { self.f = (self.f & !0x10) | if v { 0x10 } else { 0 }; }
    #[inline] pub fn get_z(&self) -> bool { (self.f & 0x80) != 0 }
    #[inline] pub fn get_n(&self) -> bool { (self.f & 0x40) != 0 }
    #[inline] pub fn get_h(&self) -> bool { (self.f & 0x20) != 0 }
    #[inline] pub fn get_cf(&self) -> bool { (self.f & 0x10) != 0 }

    fn read_imm8(&mut self, mem: &mut Memory) -> u8 {
        let v = mem.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }

    fn read_imm16(&mut self, mem: &mut Memory) -> u16 {
        let lo = mem.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        let hi = mem.read(self.pc) as u16;
        self.pc = self.pc.wrapping_add(1);
        lo | (hi << 8)
    }

    fn push(&mut self, mem: &mut Memory, val: u16) {
        self.sp = self.sp.wrapping_sub(2);
        mem.write(self.sp, (val & 0xFF) as u8);
        mem.write(self.sp.wrapping_add(1), ((val >> 8) & 0xFF) as u8);
    }

    fn pop(&mut self, mem: &mut Memory) -> u16 {
        let lo = mem.read(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        let hi = mem.read(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);
        lo | (hi << 8)
    }

    fn inc8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_add(1);
        self.set_z(r == 0);
        self.set_n(false);
        self.set_h((r & 0x0F) == 0);
        r
    }

    fn dec8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_sub(1);
        self.set_z(r == 0);
        self.set_n(true);
        self.set_h((r & 0x0F) == 0x0F);
        r
    }

    fn add_hl(&mut self, val: u16) {
        let hl = self.hl();
        let tmp = (hl as u32) + (val as u32);
        self.set_n(false);
        self.set_h((hl & 0xFFF) + (val & 0xFFF) > 0xFFF);
        self.set_cf(tmp > 0xFFFF);
        self.set_hl(tmp as u16);
    }

    fn add_a(&mut self, val: u8) {
        let r = (self.a as u16) + (val as u16);
        self.set_z((r as u8) == 0);
        self.set_n(false);
        self.set_h((self.a & 0xF) + (val & 0xF) > 0xF);
        self.set_cf(r > 0xFF);
        self.a = r as u8;
    }

    fn adc_a(&mut self, val: u8) {
        let cy = if self.get_cf() { 1u16 } else { 0 };
        let r = (self.a as u16) + (val as u16) + cy;
        self.set_z((r as u8) == 0);
        self.set_n(false);
        self.set_h((self.a & 0xF) as u16 + (val & 0xF) as u16 + cy > 0xF);
        self.set_cf(r > 0xFF);
        self.a = r as u8;
    }

    fn sub_a(&mut self, val: u8) {
        let r = (self.a as i32) - (val as i32);
        self.set_z((r as u8) == 0);
        self.set_n(true);
        self.set_h((self.a & 0xF) < (val & 0xF));
        self.set_cf(self.a < val);
        self.a = r as u8;
    }

    fn sbc_a(&mut self, val: u8) {
        let cy = if self.get_cf() { 1i32 } else { 0 };
        let r = (self.a as i32) - (val as i32) - cy;
        self.set_z((r as u8) == 0);
        self.set_n(true);
        self.set_h(((self.a & 0xF) as i32) < ((val & 0xF) as i32) + cy);
        self.set_cf(r < 0);
        self.a = r as u8;
    }

    fn and_a(&mut self, val: u8) {
        self.a &= val;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(true);
        self.set_cf(false);
    }

    fn or_a(&mut self, val: u8) {
        self.a |= val;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_cf(false);
    }

    fn xor_a(&mut self, val: u8) {
        self.a ^= val;
        self.set_z(self.a == 0);
        self.set_n(false);
        self.set_h(false);
        self.set_cf(false);
    }

    fn cp_a(&mut self, val: u8) {
        let r = (self.a as i32) - (val as i32);
        self.set_z((r as u8) == 0);
        self.set_n(true);
        self.set_h((self.a & 0xF) < (val & 0xF));
        self.set_cf(self.a < val);
    }

    fn exec_cb(&mut self, mem: &mut Memory) -> u8 {
        let op = self.read_imm8(mem);
        let r = op & 7;
        let bit = (op >> 3) & 7;
        let x = (op >> 6) & 3;

        // Read value
        let val: u8 = match r {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => mem.read(self.hl()),
            7 => self.a,
            _ => unreachable!(),
        };

        match x {
            0 => {
                let old_c = if self.get_cf() { 1u8 } else { 0 };
                let mut result = val;
                let new_c;

                match bit {
                    0 => { // RLC
                        new_c = (val >> 7) & 1;
                        result = (val << 1) | new_c;
                        self.set_z(result == 0);
                        self.set_n(false);
                        self.set_h(false);
                        self.set_cf(new_c != 0);
                    }
                    1 => { // RRC
                        new_c = val & 1;
                        result = (val >> 1) | (new_c << 7);
                        self.set_z(result == 0);
                        self.set_n(false);
                        self.set_h(false);
                        self.set_cf(new_c != 0);
                    }
                    2 => { // RL
                        new_c = (val >> 7) & 1;
                        result = (val << 1) | old_c;
                        self.set_z(result == 0);
                        self.set_n(false);
                        self.set_h(false);
                        self.set_cf(new_c != 0);
                    }
                    3 => { // RR
                        new_c = val & 1;
                        result = (val >> 1) | (old_c << 7);
                        self.set_z(result == 0);
                        self.set_n(false);
                        self.set_h(false);
                        self.set_cf(new_c != 0);
                    }
                    4 => { // SLA
                        new_c = (val >> 7) & 1;
                        result = val << 1;
                        self.set_z(result == 0);
                        self.set_n(false);
                        self.set_h(false);
                        self.set_cf(new_c != 0);
                    }
                    5 => { // SRA
                        new_c = val & 1;
                        result = (val >> 1) | (val & 0x80);
                        self.set_z(result == 0);
                        self.set_n(false);
                        self.set_h(false);
                        self.set_cf(new_c != 0);
                    }
                    6 => { // SWAP
                        result = ((val & 0x0F) << 4) | ((val >> 4) & 0x0F);
                        self.set_z(result == 0);
                        self.set_n(false);
                        self.set_h(false);
                        self.set_cf(false);
                    }
                    7 => { // SRL
                        new_c = val & 1;
                        result = val >> 1;
                        self.set_z(result == 0);
                        self.set_n(false);
                        self.set_h(false);
                        self.set_cf(new_c != 0);
                    }
                    _ => unreachable!(),
                }

                self.write_r8(mem, r, result);
                if r == 6 { 16 } else { 8 }
            }
            1 => { // BIT
                let test = (val >> bit) & 1;
                self.set_z(test == 0);
                self.set_n(false);
                self.set_h(true);
                if r == 6 { 16 } else { 8 }
            }
            2 => { // RES
                let result = val & !(1 << bit);
                self.write_r8(mem, r, result);
                if r == 6 { 16 } else { 8 }
            }
            3 => { // SET
                let result = val | (1 << bit);
                self.write_r8(mem, r, result);
                if r == 6 { 16 } else { 8 }
            }
            _ => 0,
        }
    }

    fn write_r8(&mut self, mem: &mut Memory, r: u8, val: u8) {
        match r {
            0 => self.b = val,
            1 => self.c = val,
            2 => self.d = val,
            3 => self.e = val,
            4 => self.h = val,
            5 => self.l = val,
            6 => mem.write(self.hl(), val),
            7 => self.a = val,
            _ => {}
        }
    }

    pub fn tick(&mut self, mem: &mut Memory) -> u8 {
        if self.ime_scheduled > 0 {
            self.ime_scheduled -= 1;
            if self.ime_scheduled == 0 {
                self.ime = true;
            }
        }

        let mut iflag = mem.read(IF);
        let ie_reg = mem.read(IE);

        if self.halted {
            if (iflag & ie_reg) != 0 {
                self.halted = false;
            } else {
                return 4;
            }
        }

        if self.ime && (iflag & ie_reg) != 0 {
            self.ime = false;
            self.halted = false;
            for i in 0..5 {
                if (iflag & (1 << i) & ie_reg) != 0 {
                    iflag &= !(1 << i);
                    mem.write(IF, iflag);
                    let pc = self.pc;
                    self.push(mem, pc);
                    self.pc = 0x40 + (i as u16) * 8;
                    return 20;
                }
            }
        }

        let op = self.read_imm8(mem);
        let mut cycles: u8 = 4;

        match op {
            0x00 => { cycles = 4; }
            0x01 => { let v = self.read_imm16(mem); self.set_bc(v); cycles = 12; }
            0x02 => { mem.write(self.bc(), self.a); cycles = 8; }
            0x03 => { self.set_bc(self.bc().wrapping_add(1)); cycles = 8; }
            0x04 => { self.b = self.inc8(self.b); cycles = 4; }
            0x05 => { self.b = self.dec8(self.b); cycles = 4; }
            0x06 => { self.b = self.read_imm8(mem); cycles = 8; }
            0x07 => {
                let c_bit = (self.a >> 7) & 1;
                self.a = (self.a << 1) | c_bit;
                self.set_z(false); self.set_n(false); self.set_h(false); self.set_cf(c_bit != 0);
                cycles = 4;
            }
            0x08 => {
                let addr = self.read_imm16(mem);
                mem.write(addr, (self.sp & 0xFF) as u8);
                mem.write(addr.wrapping_add(1), ((self.sp >> 8) & 0xFF) as u8);
                cycles = 20;
            }
            0x09 => { self.add_hl(self.bc()); cycles = 8; }
            0x0A => { self.a = mem.read(self.bc()); cycles = 8; }
            0x0B => { self.set_bc(self.bc().wrapping_sub(1)); cycles = 8; }
            0x0C => { self.c = self.inc8(self.c); cycles = 4; }
            0x0D => { self.c = self.dec8(self.c); cycles = 4; }
            0x0E => { self.c = self.read_imm8(mem); cycles = 8; }
            0x0F => {
                let c_bit = self.a & 1;
                self.a = (self.a >> 1) | (c_bit << 7);
                self.set_z(false); self.set_n(false); self.set_h(false); self.set_cf(c_bit != 0);
                cycles = 4;
            }

            0x10 => { self.halted = true; cycles = 4; }
            0x11 => { let v = self.read_imm16(mem); self.set_de(v); cycles = 12; }
            0x12 => { mem.write(self.de(), self.a); cycles = 8; }
            0x13 => { self.set_de(self.de().wrapping_add(1)); cycles = 8; }
            0x14 => { self.d = self.inc8(self.d); cycles = 4; }
            0x15 => { self.d = self.dec8(self.d); cycles = 4; }
            0x16 => { self.d = self.read_imm8(mem); cycles = 8; }
            0x17 => {
                let c_bit = if self.get_cf() { 1u8 } else { 0 };
                let new_c = (self.a >> 7) & 1;
                self.a = (self.a << 1) | c_bit;
                self.set_z(false); self.set_n(false); self.set_h(false); self.set_cf(new_c != 0);
                cycles = 4;
            }
            0x18 => {
                let offset = self.read_imm8(mem) as i8;
                self.pc = self.pc.wrapping_add(offset as i16 as u16);
                cycles = 12;
            }
            0x19 => { self.add_hl(self.de()); cycles = 8; }
            0x1A => { self.a = mem.read(self.de()); cycles = 8; }
            0x1B => { self.set_de(self.de().wrapping_sub(1)); cycles = 8; }
            0x1C => { self.e = self.inc8(self.e); cycles = 4; }
            0x1D => { self.e = self.dec8(self.e); cycles = 4; }
            0x1E => { self.e = self.read_imm8(mem); cycles = 8; }
            0x1F => {
                let c_bit = if self.get_cf() { 1u8 } else { 0 };
                let new_c = self.a & 1;
                self.a = (self.a >> 1) | (c_bit << 7);
                self.set_z(false); self.set_n(false); self.set_h(false); self.set_cf(new_c != 0);
                cycles = 4;
            }

            0x20 => {
                let offset = self.read_imm8(mem) as i8;
                if !self.get_z() {
                    self.pc = self.pc.wrapping_add(offset as i16 as u16);
                    cycles = 12;
                } else { cycles = 8; }
            }
            0x21 => { let v = self.read_imm16(mem); self.set_hl(v); cycles = 12; }
            0x22 => { mem.write(self.hl(), self.a); self.set_hl(self.hl().wrapping_add(1)); cycles = 8; }
            0x23 => { self.set_hl(self.hl().wrapping_add(1)); cycles = 8; }
            0x24 => { self.h = self.inc8(self.h); cycles = 4; }
            0x25 => { self.h = self.dec8(self.h); cycles = 4; }
            0x26 => { self.h = self.read_imm8(mem); cycles = 8; }
            0x27 => { // DAA
                let mut correction: u16 = 0;
                if self.get_h() || (!self.get_n() && (self.a & 0x0F) > 9) { correction |= 0x06; }
                if self.get_cf() || (!self.get_n() && self.a > 0x99) {
                    correction |= 0x60;
                    self.set_cf(true);
                }
                self.a = if self.get_n() {
                    self.a.wrapping_sub(correction as u8)
                } else {
                    self.a.wrapping_add(correction as u8)
                };
                self.set_z(self.a == 0);
                self.set_h(false);
                cycles = 4;
            }
            0x28 => {
                let offset = self.read_imm8(mem) as i8;
                if self.get_z() {
                    self.pc = self.pc.wrapping_add(offset as i16 as u16);
                    cycles = 12;
                } else { cycles = 8; }
            }
            0x29 => { self.add_hl(self.hl()); cycles = 8; }
            0x2A => { self.a = mem.read(self.hl()); self.set_hl(self.hl().wrapping_add(1)); cycles = 8; }
            0x2B => { self.set_hl(self.hl().wrapping_sub(1)); cycles = 8; }
            0x2C => { self.l = self.inc8(self.l); cycles = 4; }
            0x2D => { self.l = self.dec8(self.l); cycles = 4; }
            0x2E => { self.l = self.read_imm8(mem); cycles = 8; }
            0x2F => { self.a = !self.a; self.set_n(true); self.set_h(true); cycles = 4; }

            0x30 => {
                let offset = self.read_imm8(mem) as i8;
                if !self.get_cf() {
                    self.pc = self.pc.wrapping_add(offset as i16 as u16);
                    cycles = 12;
                } else { cycles = 8; }
            }
            0x31 => { self.sp = self.read_imm16(mem); cycles = 12; }
            0x32 => { mem.write(self.hl(), self.a); self.set_hl(self.hl().wrapping_sub(1)); cycles = 8; }
            0x33 => { self.sp = self.sp.wrapping_add(1); cycles = 8; }
            0x34 => {
                let mut val = mem.read(self.hl());
                val = val.wrapping_add(1);
                self.set_z(val == 0); self.set_n(false); self.set_h((val & 0x0F) == 0);
                mem.write(self.hl(), val);
                cycles = 12;
            }
            0x35 => {
                let mut val = mem.read(self.hl());
                val = val.wrapping_sub(1);
                self.set_z(val == 0); self.set_n(true); self.set_h((val & 0x0F) == 0x0F);
                mem.write(self.hl(), val);
                cycles = 12;
            }
            0x36 => { let v = self.read_imm8(mem); mem.write(self.hl(), v); cycles = 12; }
            0x37 => { self.set_cf(true); self.set_n(false); self.set_h(false); cycles = 4; }
            0x38 => {
                let offset = self.read_imm8(mem) as i8;
                if self.get_cf() {
                    self.pc = self.pc.wrapping_add(offset as i16 as u16);
                    cycles = 12;
                } else { cycles = 8; }
            }
            0x39 => { self.add_hl(self.sp); cycles = 8; }
            0x3A => { self.a = mem.read(self.hl()); self.set_hl(self.hl().wrapping_sub(1)); cycles = 8; }
            0x3B => { self.sp = self.sp.wrapping_sub(1); cycles = 8; }
            0x3C => { self.a = self.inc8(self.a); cycles = 4; }
            0x3D => { self.a = self.dec8(self.a); cycles = 4; }
            0x3E => { self.a = self.read_imm8(mem); cycles = 8; }
            0x3F => { self.set_cf(!self.get_cf()); self.set_n(false); self.set_h(false); cycles = 4; }

            // 0x40-0x7F LD r,r
            0x40 => { cycles = 4; }
            0x41 => { self.b = self.c; cycles = 4; }
            0x42 => { self.b = self.d; cycles = 4; }
            0x43 => { self.b = self.e; cycles = 4; }
            0x44 => { self.b = self.h; cycles = 4; }
            0x45 => { self.b = self.l; cycles = 4; }
            0x46 => { self.b = mem.read(self.hl()); cycles = 8; }
            0x47 => { self.b = self.a; cycles = 4; }
            0x48 => { self.c = self.b; cycles = 4; }
            0x49 => { cycles = 4; }
            0x4A => { self.c = self.d; cycles = 4; }
            0x4B => { self.c = self.e; cycles = 4; }
            0x4C => { self.c = self.h; cycles = 4; }
            0x4D => { self.c = self.l; cycles = 4; }
            0x4E => { self.c = mem.read(self.hl()); cycles = 8; }
            0x4F => { self.c = self.a; cycles = 4; }

            0x50 => { self.d = self.b; cycles = 4; }
            0x51 => { self.d = self.c; cycles = 4; }
            0x52 => { cycles = 4; }
            0x53 => { self.d = self.e; cycles = 4; }
            0x54 => { self.d = self.h; cycles = 4; }
            0x55 => { self.d = self.l; cycles = 4; }
            0x56 => { self.d = mem.read(self.hl()); cycles = 8; }
            0x57 => { self.d = self.a; cycles = 4; }
            0x58 => { self.e = self.b; cycles = 4; }
            0x59 => { self.e = self.c; cycles = 4; }
            0x5A => { self.e = self.d; cycles = 4; }
            0x5B => { cycles = 4; }
            0x5C => { self.e = self.h; cycles = 4; }
            0x5D => { self.e = self.l; cycles = 4; }
            0x5E => { self.e = mem.read(self.hl()); cycles = 8; }
            0x5F => { self.e = self.a; cycles = 4; }

            0x60 => { self.h = self.b; cycles = 4; }
            0x61 => { self.h = self.c; cycles = 4; }
            0x62 => { self.h = self.d; cycles = 4; }
            0x63 => { self.h = self.e; cycles = 4; }
            0x64 => { cycles = 4; }
            0x65 => { self.h = self.l; cycles = 4; }
            0x66 => { self.h = mem.read(self.hl()); cycles = 8; }
            0x67 => { self.h = self.a; cycles = 4; }
            0x68 => { self.l = self.b; cycles = 4; }
            0x69 => { self.l = self.c; cycles = 4; }
            0x6A => { self.l = self.d; cycles = 4; }
            0x6B => { self.l = self.e; cycles = 4; }
            0x6C => { self.l = self.h; cycles = 4; }
            0x6D => { cycles = 4; }
            0x6E => { self.l = mem.read(self.hl()); cycles = 8; }
            0x6F => { self.l = self.a; cycles = 4; }

            0x70 => { mem.write(self.hl(), self.b); cycles = 8; }
            0x71 => { mem.write(self.hl(), self.c); cycles = 8; }
            0x72 => { mem.write(self.hl(), self.d); cycles = 8; }
            0x73 => { mem.write(self.hl(), self.e); cycles = 8; }
            0x74 => { mem.write(self.hl(), self.h); cycles = 8; }
            0x75 => { mem.write(self.hl(), self.l); cycles = 8; }
            0x76 => { self.halted = true; cycles = 4; }
            0x77 => { mem.write(self.hl(), self.a); cycles = 8; }

            0x78 => { self.a = self.b; cycles = 4; }
            0x79 => { self.a = self.c; cycles = 4; }
            0x7A => { self.a = self.d; cycles = 4; }
            0x7B => { self.a = self.e; cycles = 4; }
            0x7C => { self.a = self.h; cycles = 4; }
            0x7D => { self.a = self.l; cycles = 4; }
            0x7E => { self.a = mem.read(self.hl()); cycles = 8; }
            0x7F => { cycles = 4; }

            // ADD A,r
            0x80 => { self.add_a(self.b); cycles = 4; }
            0x81 => { self.add_a(self.c); cycles = 4; }
            0x82 => { self.add_a(self.d); cycles = 4; }
            0x83 => { self.add_a(self.e); cycles = 4; }
            0x84 => { self.add_a(self.h); cycles = 4; }
            0x85 => { self.add_a(self.l); cycles = 4; }
            0x86 => { let v = mem.read(self.hl()); self.add_a(v); cycles = 8; }
            0x87 => { self.add_a(self.a); cycles = 4; }

            // ADC
            0x88 => { self.adc_a(self.b); cycles = 4; }
            0x89 => { self.adc_a(self.c); cycles = 4; }
            0x8A => { self.adc_a(self.d); cycles = 4; }
            0x8B => { self.adc_a(self.e); cycles = 4; }
            0x8C => { self.adc_a(self.h); cycles = 4; }
            0x8D => { self.adc_a(self.l); cycles = 4; }
            0x8E => { let v = mem.read(self.hl()); self.adc_a(v); cycles = 8; }
            0x8F => { self.adc_a(self.a); cycles = 4; }

            // SUB
            0x90 => { self.sub_a(self.b); cycles = 4; }
            0x91 => { self.sub_a(self.c); cycles = 4; }
            0x92 => { self.sub_a(self.d); cycles = 4; }
            0x93 => { self.sub_a(self.e); cycles = 4; }
            0x94 => { self.sub_a(self.h); cycles = 4; }
            0x95 => { self.sub_a(self.l); cycles = 4; }
            0x96 => { let v = mem.read(self.hl()); self.sub_a(v); cycles = 8; }
            0x97 => { self.set_z(true); self.set_n(true); self.set_h(false); self.set_cf(false); self.a = 0; cycles = 4; }

            // SBC
            0x98 => { self.sbc_a(self.b); cycles = 4; }
            0x99 => { self.sbc_a(self.c); cycles = 4; }
            0x9A => { self.sbc_a(self.d); cycles = 4; }
            0x9B => { self.sbc_a(self.e); cycles = 4; }
            0x9C => { self.sbc_a(self.h); cycles = 4; }
            0x9D => { self.sbc_a(self.l); cycles = 4; }
            0x9E => { let v = mem.read(self.hl()); self.sbc_a(v); cycles = 8; }
            0x9F => { self.sbc_a(self.a); cycles = 4; }

            // AND
            0xA0 => { self.and_a(self.b); cycles = 4; }
            0xA1 => { self.and_a(self.c); cycles = 4; }
            0xA2 => { self.and_a(self.d); cycles = 4; }
            0xA3 => { self.and_a(self.e); cycles = 4; }
            0xA4 => { self.and_a(self.h); cycles = 4; }
            0xA5 => { self.and_a(self.l); cycles = 4; }
            0xA6 => { let v = mem.read(self.hl()); self.and_a(v); cycles = 8; }
            0xA7 => { self.and_a(self.a); cycles = 4; }

            // XOR
            0xA8 => { self.xor_a(self.b); cycles = 4; }
            0xA9 => { self.xor_a(self.c); cycles = 4; }
            0xAA => { self.xor_a(self.d); cycles = 4; }
            0xAB => { self.xor_a(self.e); cycles = 4; }
            0xAC => { self.xor_a(self.h); cycles = 4; }
            0xAD => { self.xor_a(self.l); cycles = 4; }
            0xAE => { let v = mem.read(self.hl()); self.xor_a(v); cycles = 8; }
            0xAF => { self.a = 0; self.set_z(true); self.set_n(false); self.set_h(false); self.set_cf(false); cycles = 4; }

            // OR
            0xB0 => { self.or_a(self.b); cycles = 4; }
            0xB1 => { self.or_a(self.c); cycles = 4; }
            0xB2 => { self.or_a(self.d); cycles = 4; }
            0xB3 => { self.or_a(self.e); cycles = 4; }
            0xB4 => { self.or_a(self.h); cycles = 4; }
            0xB5 => { self.or_a(self.l); cycles = 4; }
            0xB6 => { let v = mem.read(self.hl()); self.or_a(v); cycles = 8; }
            0xB7 => { self.or_a(self.a); cycles = 4; }

            // CP
            0xB8 => { self.cp_a(self.b); cycles = 4; }
            0xB9 => { self.cp_a(self.c); cycles = 4; }
            0xBA => { self.cp_a(self.d); cycles = 4; }
            0xBB => { self.cp_a(self.e); cycles = 4; }
            0xBC => { self.cp_a(self.h); cycles = 4; }
            0xBD => { self.cp_a(self.l); cycles = 4; }
            0xBE => { let v = mem.read(self.hl()); self.cp_a(v); cycles = 8; }
            0xBF => { self.set_z(true); self.set_n(true); self.set_h(false); self.set_cf(false); cycles = 4; }

            0xC0 => {
                if !self.get_z() { self.pc = self.pop(mem); cycles = 20; }
                else { cycles = 8; }
            }
            0xC1 => { let v = self.pop(mem); self.set_bc(v); cycles = 12; }
            0xC2 => {
                let addr = self.read_imm16(mem);
                if !self.get_z() { self.pc = addr; cycles = 16; } else { cycles = 12; }
            }
            0xC3 => { self.pc = self.read_imm16(mem); cycles = 16; }
            0xC4 => {
                let addr = self.read_imm16(mem);
                if !self.get_z() { let pc = self.pc; self.push(mem, pc); self.pc = addr; cycles = 24; }
                else { cycles = 12; }
            }
            0xC5 => { let v = self.bc(); self.push(mem, v); cycles = 16; }
            0xC6 => { let v = self.read_imm8(mem); self.add_a(v); cycles = 8; }
            0xC7 => { let pc = self.pc; self.push(mem, pc); self.pc = 0x00; cycles = 16; }
            0xC8 => {
                if self.get_z() { self.pc = self.pop(mem); cycles = 20; }
                else { cycles = 8; }
            }
            0xC9 => { self.pc = self.pop(mem); cycles = 16; }
            0xCA => {
                let addr = self.read_imm16(mem);
                if self.get_z() { self.pc = addr; cycles = 16; } else { cycles = 12; }
            }
            0xCB => { cycles = self.exec_cb(mem); }
            0xCC => {
                let addr = self.read_imm16(mem);
                if self.get_z() { let pc = self.pc; self.push(mem, pc); self.pc = addr; cycles = 24; }
                else { cycles = 12; }
            }
            0xCD => { let addr = self.read_imm16(mem); let pc = self.pc; self.push(mem, pc); self.pc = addr; cycles = 24; }
            0xCE => { let v = self.read_imm8(mem); self.adc_a(v); cycles = 8; }
            0xCF => { let pc = self.pc; self.push(mem, pc); self.pc = 0x08; cycles = 16; }

            0xD0 => {
                if !self.get_cf() { self.pc = self.pop(mem); cycles = 20; }
                else { cycles = 8; }
            }
            0xD1 => { let v = self.pop(mem); self.set_de(v); cycles = 12; }
            0xD2 => {
                let addr = self.read_imm16(mem);
                if !self.get_cf() { self.pc = addr; cycles = 16; } else { cycles = 12; }
            }
            0xD4 => {
                let addr = self.read_imm16(mem);
                if !self.get_cf() { let pc = self.pc; self.push(mem, pc); self.pc = addr; cycles = 24; }
                else { cycles = 12; }
            }
            0xD5 => { let v = self.de(); self.push(mem, v); cycles = 16; }
            0xD6 => { let v = self.read_imm8(mem); self.sub_a(v); cycles = 8; }
            0xD7 => { let pc = self.pc; self.push(mem, pc); self.pc = 0x10; cycles = 16; }
            0xD8 => {
                if self.get_cf() { self.pc = self.pop(mem); cycles = 20; }
                else { cycles = 8; }
            }
            0xD9 => { self.pc = self.pop(mem); self.ime = true; cycles = 16; }
            0xDA => {
                let addr = self.read_imm16(mem);
                if self.get_cf() { self.pc = addr; cycles = 16; } else { cycles = 12; }
            }
            0xDC => {
                let addr = self.read_imm16(mem);
                if self.get_cf() { let pc = self.pc; self.push(mem, pc); self.pc = addr; cycles = 24; }
                else { cycles = 12; }
            }
            0xDE => { let v = self.read_imm8(mem); self.sbc_a(v); cycles = 8; }
            0xDF => { let pc = self.pc; self.push(mem, pc); self.pc = 0x18; cycles = 16; }

            0xE0 => { let offset = self.read_imm8(mem); mem.write(0xFF00u16.wrapping_add(offset as u16), self.a); cycles = 12; }
            0xE1 => { let v = self.pop(mem); self.set_hl(v); cycles = 12; }
            0xE2 => { mem.write(0xFF00u16.wrapping_add(self.c as u16), self.a); cycles = 8; }
            0xE5 => { let v = self.hl(); self.push(mem, v); cycles = 16; }
            0xE6 => { let v = self.read_imm8(mem); self.and_a(v); cycles = 8; }
            0xE7 => { let pc = self.pc; self.push(mem, pc); self.pc = 0x20; cycles = 16; }
            0xE8 => {
                let offset = self.read_imm8(mem) as i8 as i16;
                let sp = self.sp;
                let r = sp.wrapping_add(offset as u16);
                self.set_z(false);
                self.set_n(false);
                self.set_h((sp & 0xF) + ((offset as u16) & 0xF) > 0xF);
                self.set_cf((sp & 0xFF) + ((offset as u16) & 0xFF) > 0xFF);
                self.sp = r;
                cycles = 16;
            }
            0xE9 => { self.pc = self.hl(); cycles = 4; }
            0xEA => { let addr = self.read_imm16(mem); mem.write(addr, self.a); cycles = 16; }
            0xEE => { let v = self.read_imm8(mem); self.xor_a(v); cycles = 8; }
            0xEF => { let pc = self.pc; self.push(mem, pc); self.pc = 0x28; cycles = 16; }

            0xF0 => { let offset = self.read_imm8(mem); self.a = mem.read(0xFF00u16.wrapping_add(offset as u16)); cycles = 12; }
            0xF1 => { let v = self.pop(mem); self.set_af(v & 0xFFF0); cycles = 12; }
            0xF2 => { self.a = mem.read(0xFF00u16.wrapping_add(self.c as u16)); cycles = 8; }
            0xF3 => { self.ime = false; self.ime_scheduled = 0; cycles = 4; }
            0xF5 => { let v = self.af(); self.push(mem, v); cycles = 16; }
            0xF6 => { let v = self.read_imm8(mem); self.or_a(v); cycles = 8; }
            0xF7 => { let pc = self.pc; self.push(mem, pc); self.pc = 0x30; cycles = 16; }
            0xF8 => {
                let offset = self.read_imm8(mem) as i8 as i16;
                let sp = self.sp;
                let r = sp.wrapping_add(offset as u16);
                self.set_z(false);
                self.set_n(false);
                self.set_h((sp & 0xF) + ((offset as u16) & 0xF) > 0xF);
                self.set_cf((sp & 0xFF) + ((offset as u16) & 0xFF) > 0xFF);
                self.set_hl(r);
                cycles = 12;
            }
            0xF9 => { self.sp = self.hl(); cycles = 8; }
            0xFA => { let addr = self.read_imm16(mem); self.a = mem.read(addr); cycles = 16; }
            0xFB => { self.ime_scheduled = 1; cycles = 4; }
            0xFE => { let v = self.read_imm8(mem); self.cp_a(v); cycles = 8; }
            0xFF => { let pc = self.pc; self.push(mem, pc); self.pc = 0x38; cycles = 16; }

            _ => { cycles = 4; }
        }

        cycles
    }
}
