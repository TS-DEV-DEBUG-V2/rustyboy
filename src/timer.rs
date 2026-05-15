use crate::types::*;
use crate::memory::Memory;

pub struct Timer {
    pub div_counter: u16,
    pub tima_counter: u16,
    pub prev_div_bit: u32,
}

impl Timer {
    pub fn new() -> Self {
        Self { div_counter: 0, tima_counter: 0, prev_div_bit: 0 }
    }

    pub fn reset(&mut self) {
        self.div_counter = 0;
        self.tima_counter = 0;
        self.prev_div_bit = 0;
    }

    pub fn step(&mut self, cycles: u8, mem: &mut Memory) {
        for _ in 0..cycles {
            self.div_counter = self.div_counter.wrapping_add(1);
            if self.div_counter >= 256 {
                self.div_counter = 0;
                let div = mem.read(DIV);
                mem.write(DIV, div.wrapping_add(1));
            }

            let tac = mem.read(TAC);
            if (tac & 0x04) != 0 {
                let clock_select = tac & 0x03;
                let div_bit: u32 = match clock_select {
                    0 => if (self.div_counter & (1 << 9)) != 0 { 1 } else { 0 },
                    1 => if (self.div_counter & (1 << 3)) != 0 { 1 } else { 0 },
                    2 => if (self.div_counter & (1 << 5)) != 0 { 1 } else { 0 },
                    3 => if (self.div_counter & (1 << 7)) != 0 { 1 } else { 0 },
                    _ => 0,
                };

                if self.prev_div_bit == 1 && div_bit == 0 {
                    let mut tima = mem.read(TIMA);
                    tima = tima.wrapping_add(1);
                    if tima == 0 {
                        tima = mem.read(TMA);
                        let mut iflag = mem.read(IF);
                        iflag |= INT_TIMER;
                        mem.write(IF, iflag);
                    }
                    mem.write(TIMA, tima);
                }
                self.prev_div_bit = div_bit;
            }
        }
    }
}
