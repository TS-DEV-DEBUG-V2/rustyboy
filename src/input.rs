pub const BTN_RIGHT:  usize = 0;
pub const BTN_LEFT:   usize = 1;
pub const BTN_UP:     usize = 2;
pub const BTN_DOWN:   usize = 3;
pub const BTN_A:      usize = 4;
pub const BTN_B:      usize = 5;
pub const BTN_SELECT: usize = 6;
pub const BTN_START:  usize = 7;

pub struct Input {
    pub buttons: [bool; 8],
    pub p1_state: u8,
}

impl Input {
    pub fn new() -> Self {
        Self {
            buttons: [false; 8],
            p1_state: 0xFF,
        }
    }

    pub fn key_down(&mut self, b: usize) {
        if !self.buttons[b] {
            self.buttons[b] = true;
        }
    }

    pub fn key_up(&mut self, b: usize) {
        self.buttons[b] = false;
    }

    pub fn read_p1(&self) -> u8 {
        let mut result: u8 = 0xFF;
        let select = self.p1_state & 0x30;

        if (select & 0x10) == 0 {
            if self.buttons[BTN_RIGHT] { result &= !(1 << 0); }
            if self.buttons[BTN_LEFT]  { result &= !(1 << 1); }
            if self.buttons[BTN_UP]    { result &= !(1 << 2); }
            if self.buttons[BTN_DOWN]  { result &= !(1 << 3); }
        }
        if (select & 0x20) == 0 {
            if self.buttons[BTN_A]      { result &= !(1 << 0); }
            if self.buttons[BTN_B]      { result &= !(1 << 1); }
            if self.buttons[BTN_SELECT] { result &= !(1 << 2); }
            if self.buttons[BTN_START]  { result &= !(1 << 3); }
        }

        (result & 0x0F) | (self.p1_state & 0x30) | 0xC0
    }

    pub fn write_p1(&mut self, val: u8) {
        self.p1_state = (self.p1_state & 0xCF) | (val & 0x30);
    }
}
