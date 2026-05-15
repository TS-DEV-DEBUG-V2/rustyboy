use crate::types::*;

pub const SAMPLE_RATE: i32 = 44100;
pub const CPU_CLOCK: i32 = 4194304;
pub const AUDIO_BUFFER_SIZE: usize = 4096;

const DUTY_WAVES: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 0, 1, 1, 1],
    [0, 1, 1, 1, 1, 1, 1, 0],
];

const NOISE_DIVISORS: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

#[derive(Default, Clone)]
pub struct SquareChannel {
    pub enabled: bool,
    pub duty: u8,
    pub duty_pos: u8,
    pub volume: u8,
    pub envelope_dir: u8,
    pub envelope_period: u8,
    pub envelope_counter: i32,
    pub frequency: u16,
    pub freq_counter: i32,
    pub length_counter: u16,
    pub sweep_period: u8,
    pub sweep_shift: u8,
    pub sweep_dir: u8,
    pub sweep_counter: i32,
    pub sweep_enabled: bool,
    pub shadow_freq: u16,
    pub length_enable: bool,
    pub dac_enabled: bool,
}

#[derive(Default, Clone)]
pub struct WaveChannel {
    pub enabled: bool,
    pub dac_enabled: bool,
    pub volume_code: u8,
    pub length_counter: u16,
    pub frequency: u16,
    pub freq_counter: i32,
    pub sample_pos: u8,
    pub length_enable: bool,
}

#[derive(Default, Clone)]
pub struct NoiseChannel {
    pub enabled: bool,
    pub volume: u8,
    pub envelope_dir: u8,
    pub envelope_period: u8,
    pub envelope_counter: i32,
    pub lfsr: u16,
    pub divisor_code: u8,
    pub width_mode: u8,
    pub shift_amount: u8,
    pub freq_counter: i32,
    pub length_counter: u16,
    pub length_enable: bool,
    pub dac_enabled: bool,
}

pub struct Apu {
    pub audio_buffer: Vec<i16>,
    pub buffer_pos: usize,
    pub frame_seq_cycle: i32,
    pub frame_seq_step: i32,
    pub sample_counter: i32,
    pub ch1: SquareChannel,
    pub ch2: SquareChannel,
    pub ch3: WaveChannel,
    pub ch4: NoiseChannel,
}

impl Apu {
    pub fn new() -> Self {
        let mut a = Self {
            audio_buffer: vec![0i16; AUDIO_BUFFER_SIZE],
            buffer_pos: 0,
            frame_seq_cycle: 0,
            frame_seq_step: 0,
            sample_counter: 0,
            ch1: SquareChannel::default(),
            ch2: SquareChannel::default(),
            ch3: WaveChannel::default(),
            ch4: NoiseChannel::default(),
        };
        a.ch4.lfsr = 0x7FFF;
        a
    }

    pub fn reset(&mut self) {
        self.buffer_pos = 0;
        self.frame_seq_cycle = 0;
        self.frame_seq_step = 0;
        self.sample_counter = 0;
        self.ch1 = SquareChannel::default();
        self.ch2 = SquareChannel::default();
        self.ch3 = WaveChannel::default();
        self.ch4 = NoiseChannel::default();
        self.ch4.lfsr = 0x7FFF;
    }

    fn noise_period(&self) -> u16 {
        let base = NOISE_DIVISORS[(self.ch4.divisor_code & 7) as usize];
        base << self.ch4.shift_amount
    }

    pub fn write_reg(&mut self, addr: u16, val: u8) {
        match addr {
            NR10 => {
                self.ch1.sweep_period = (val >> 4) & 7;
                self.ch1.sweep_dir = (val >> 3) & 1;
                self.ch1.sweep_shift = val & 7;
            }
            NR11 => {
                self.ch1.duty = (val >> 6) & 3;
                self.ch1.length_counter = 64 - (val as u16 & 63);
            }
            NR12 => {
                self.ch1.volume = (val >> 4) & 0xF;
                self.ch1.envelope_dir = (val >> 3) & 1;
                self.ch1.envelope_period = val & 7;
                self.ch1.dac_enabled = (val & 0xF8) != 0;
                if !self.ch1.dac_enabled { self.ch1.enabled = false; }
            }
            NR13 => {
                self.ch1.frequency = (self.ch1.frequency & 0x700) | val as u16;
            }
            NR14 => {
                self.ch1.frequency = (self.ch1.frequency & 0xFF) | (((val & 7) as u16) << 8);
                self.ch1.length_enable = ((val >> 6) & 1) != 0;
            }

            NR21 => {
                self.ch2.duty = (val >> 6) & 3;
                self.ch2.length_counter = 64 - (val as u16 & 63);
            }
            NR22 => {
                self.ch2.volume = (val >> 4) & 0xF;
                self.ch2.envelope_dir = (val >> 3) & 1;
                self.ch2.envelope_period = val & 7;
                self.ch2.dac_enabled = (val & 0xF8) != 0;
                if !self.ch2.dac_enabled { self.ch2.enabled = false; }
            }
            NR23 => {
                self.ch2.frequency = (self.ch2.frequency & 0x700) | val as u16;
            }
            NR24 => {
                self.ch2.frequency = (self.ch2.frequency & 0xFF) | (((val & 7) as u16) << 8);
                self.ch2.length_enable = ((val >> 6) & 1) != 0;
            }

            NR30 => {
                self.ch3.dac_enabled = (val & 0x80) != 0;
                if !self.ch3.dac_enabled { self.ch3.enabled = false; }
            }
            NR31 => {
                self.ch3.length_counter = 256 - val as u16;
            }
            NR32 => {
                self.ch3.volume_code = (val >> 5) & 3;
            }
            NR33 => {
                self.ch3.frequency = (self.ch3.frequency & 0x700) | val as u16;
            }
            NR34 => {
                self.ch3.frequency = (self.ch3.frequency & 0xFF) | (((val & 7) as u16) << 8);
                self.ch3.length_enable = ((val >> 6) & 1) != 0;
            }

            NR41 => {
                self.ch4.length_counter = 64 - (val as u16 & 63);
            }
            NR42 => {
                self.ch4.volume = (val >> 4) & 0xF;
                self.ch4.envelope_dir = (val >> 3) & 1;
                self.ch4.envelope_period = val & 7;
                self.ch4.dac_enabled = (val & 0xF8) != 0;
                if !self.ch4.dac_enabled { self.ch4.enabled = false; }
            }
            NR43 => {
                self.ch4.divisor_code = val & 7;
                self.ch4.width_mode = (val >> 3) & 1;
                self.ch4.shift_amount = (val >> 4) & 0xF;
            }
            NR44 => {
                self.ch4.length_enable = ((val >> 6) & 1) != 0;
            }

            _ => {}
        }

        if addr == NR14 && (val & 0x80) != 0 { self.trigger_ch1(); }
        if addr == NR24 && (val & 0x80) != 0 { self.trigger_ch2(); }
        if addr == NR34 && (val & 0x80) != 0 { self.trigger_ch3(); }
        if addr == NR44 && (val & 0x80) != 0 { self.trigger_ch4(); }
    }

    fn trigger_ch1(&mut self) {
        self.ch1.enabled = self.ch1.dac_enabled;
        self.ch1.freq_counter = (2048i32 - self.ch1.frequency as i32) * 4;
        self.ch1.duty_pos = 0;
        self.ch1.envelope_counter = self.ch1.envelope_period as i32;
        if self.ch1.length_counter == 0 { self.ch1.length_counter = 64; }
        self.ch1.shadow_freq = self.ch1.frequency;
        self.ch1.sweep_counter = if self.ch1.sweep_period != 0 {
            self.ch1.sweep_period as i32
        } else {
            8
        };
        self.ch1.sweep_enabled = self.ch1.sweep_period > 0 || self.ch1.sweep_shift > 0;
        if self.ch1.sweep_shift > 0 {
            let delta = self.ch1.shadow_freq >> self.ch1.sweep_shift;
            let new_freq = if self.ch1.sweep_dir != 0 {
                self.ch1.shadow_freq.wrapping_sub(delta)
            } else {
                self.ch1.shadow_freq.wrapping_add(delta)
            };
            if new_freq > 2047 { self.ch1.enabled = false; }
        }
    }

    fn trigger_ch2(&mut self) {
        self.ch2.enabled = self.ch2.dac_enabled;
        self.ch2.freq_counter = (2048i32 - self.ch2.frequency as i32) * 4;
        self.ch2.duty_pos = 0;
        self.ch2.envelope_counter = self.ch2.envelope_period as i32;
        if self.ch2.length_counter == 0 { self.ch2.length_counter = 64; }
    }

    fn trigger_ch3(&mut self) {
        self.ch3.enabled = self.ch3.dac_enabled;
        self.ch3.freq_counter = (2048i32 - self.ch3.frequency as i32) * 2;
        self.ch3.sample_pos = 0;
        if self.ch3.length_counter == 0 { self.ch3.length_counter = 256; }
    }

    fn trigger_ch4(&mut self) {
        self.ch4.enabled = self.ch4.dac_enabled;
        self.ch4.freq_counter = self.noise_period() as i32;
        self.ch4.lfsr = 0x7FFF;
        self.ch4.envelope_counter = self.ch4.envelope_period as i32;
        if self.ch4.length_counter == 0 { self.ch4.length_counter = 64; }
    }

    fn tick_square(ch: &mut SquareChannel) {
        if !ch.enabled || !ch.dac_enabled { return; }
        ch.freq_counter -= 1;
        if ch.freq_counter <= 0 {
            ch.freq_counter = (2048i32 - ch.frequency as i32) * 4;
            ch.duty_pos = (ch.duty_pos + 1) & 7;
        }
    }

    fn tick_wave(ch: &mut WaveChannel) {
        if !ch.enabled || !ch.dac_enabled { return; }
        ch.freq_counter -= 1;
        if ch.freq_counter <= 0 {
            ch.freq_counter = (2048i32 - ch.frequency as i32) * 2;
            ch.sample_pos = (ch.sample_pos + 1) & 31;
        }
    }

    fn tick_noise(ch: &mut NoiseChannel) {
        if !ch.enabled || !ch.dac_enabled { return; }
        ch.freq_counter -= 1;
        if ch.freq_counter <= 0 {
            ch.freq_counter = (NOISE_DIVISORS[(ch.divisor_code & 7) as usize] << ch.shift_amount) as i32;
            let xor_r = (ch.lfsr & 1) ^ ((ch.lfsr >> 1) & 1);
            ch.lfsr >>= 1;
            if xor_r != 0 { ch.lfsr |= 0x4000; }
            if ch.width_mode != 0 {
                ch.lfsr &= !0x40;
                if xor_r != 0 { ch.lfsr |= 0x40; }
                ch.lfsr &= 0x407F;
            }
        }
    }

    fn read_square(ch: &SquareChannel) -> f32 {
        if !ch.enabled || !ch.dac_enabled { return 0.0; }
        let wave_out = DUTY_WAVES[ch.duty as usize][ch.duty_pos as usize];
        let dac_in = if wave_out != 0 { ch.volume as f32 } else { 0.0 };
        dac_in / 7.5 - 1.0
    }

    fn read_wave(ch: &WaveChannel, io_regs: &[u8; 0x80]) -> f32 {
        if !ch.enabled || !ch.dac_enabled { return 0.0; }
        let addr_off = (0xFF30 + (ch.sample_pos as usize) / 2) - 0xFF00;
        let byte = io_regs[addr_off];
        let nibble = if (ch.sample_pos & 1) != 0 {
            byte & 0x0F
        } else {
            byte >> 4
        };
        let shifted = match ch.volume_code {
            0 => 0u8,
            1 => nibble,
            2 => nibble >> 1,
            3 => nibble >> 2,
            _ => 0,
        };
        (shifted as f32) / 7.5 - 1.0
    }

    fn read_noise(ch: &NoiseChannel) -> f32 {
        if !ch.enabled || !ch.dac_enabled { return 0.0; }
        let dac_in = if (ch.lfsr & 1) == 0 { ch.volume as f32 } else { 0.0 };
        dac_in / 7.5 - 1.0
    }

    fn tick_frame_sequencer(&mut self) {
        self.frame_seq_step = (self.frame_seq_step + 1) & 7;

        if (self.frame_seq_step & 1) == 0 {
            tick_length_sq(&mut self.ch1);
            tick_length_sq(&mut self.ch2);
            tick_length_wave(&mut self.ch3);
            tick_length_noise(&mut self.ch4);
        }

        if self.frame_seq_step == 2 || self.frame_seq_step == 6 {
            if self.ch1.sweep_enabled && self.ch1.sweep_period > 0 {
                self.ch1.sweep_counter -= 1;
                if self.ch1.sweep_counter <= 0 {
                    self.ch1.sweep_counter = if self.ch1.sweep_period != 0 {
                        self.ch1.sweep_period as i32
                    } else {
                        8
                    };
                    let delta = self.ch1.shadow_freq >> self.ch1.sweep_shift;
                    let new_freq = if self.ch1.sweep_dir != 0 {
                        self.ch1.shadow_freq.wrapping_sub(delta)
                    } else {
                        self.ch1.shadow_freq.wrapping_add(delta)
                    };
                    if new_freq > 2047 {
                        self.ch1.enabled = false;
                    } else if self.ch1.sweep_shift > 0 {
                        self.ch1.frequency = new_freq;
                        self.ch1.shadow_freq = new_freq;
                        let delta2 = new_freq >> self.ch1.sweep_shift;
                        let check = if self.ch1.sweep_dir != 0 {
                            new_freq.wrapping_sub(delta2)
                        } else {
                            new_freq.wrapping_add(delta2)
                        };
                        if check > 2047 { self.ch1.enabled = false; }
                    }
                }
            }
        }

        if self.frame_seq_step == 7 {
            tick_env_sq(&mut self.ch1);
            tick_env_sq(&mut self.ch2);
            tick_env_noise(&mut self.ch4);
        }
    }

    fn generate_sample(&mut self, io_regs: &[u8; 0x80]) {
        if self.buffer_pos >= self.audio_buffer.len() { return; }

        let nr50 = io_regs[(NR50 as usize) - 0xFF00];
        let nr51 = io_regs[(NR51 as usize) - 0xFF00];

        let ch1_out = Self::read_square(&self.ch1);
        let ch2_out = Self::read_square(&self.ch2);
        let ch3_out = Self::read_wave(&self.ch3, io_regs);
        let ch4_out = Self::read_noise(&self.ch4);

        let mut left = 0.0f32;
        let mut right = 0.0f32;

        if (nr51 & 0x10) != 0 { left  += ch1_out; }
        if (nr51 & 0x01) != 0 { right += ch1_out; }
        if (nr51 & 0x20) != 0 { left  += ch2_out; }
        if (nr51 & 0x02) != 0 { right += ch2_out; }
        if (nr51 & 0x40) != 0 { left  += ch3_out; }
        if (nr51 & 0x04) != 0 { right += ch3_out; }
        if (nr51 & 0x80) != 0 { left  += ch4_out; }
        if (nr51 & 0x08) != 0 { right += ch4_out; }

        let left_vol  = ((nr50 >> 4) & 7) as f32 + 1.0;
        let right_vol = (nr50 & 7) as f32 + 1.0;
        left  *= left_vol / 8.0;
        right *= right_vol / 8.0;

        left  *= 0.25;
        right *= 0.25;

        let mut mono = (left + right) * 0.5;
        if mono > 1.0 { mono = 1.0; }
        if mono < -1.0 { mono = -1.0; }

        self.audio_buffer[self.buffer_pos] = (mono * 32767.0) as i16;
        self.buffer_pos += 1;
    }

    fn update_nr52(&self, io_regs: &mut [u8; 0x80]) {
        let nr52 = io_regs[(NR52 as usize) - 0xFF00];
        if (nr52 & 0x80) != 0 {
            let mut status: u8 = 0x80;
            if self.ch1.enabled && self.ch1.dac_enabled { status |= 1; }
            if self.ch2.enabled && self.ch2.dac_enabled { status |= 2; }
            if self.ch3.enabled && self.ch3.dac_enabled { status |= 4; }
            if self.ch4.enabled && self.ch4.dac_enabled { status |= 8; }
            io_regs[(NR52 as usize) - 0xFF00] = status;
        }
    }

    pub fn step(&mut self, cycles: u8, io_regs: &mut [u8; 0x80]) {
        let nr52 = io_regs[(NR52 as usize) - 0xFF00];
        if (nr52 & 0x80) == 0 { return; }

        for _ in 0..cycles {
            Self::tick_square(&mut self.ch1);
            Self::tick_square(&mut self.ch2);
            Self::tick_wave(&mut self.ch3);
            Self::tick_noise(&mut self.ch4);

            self.frame_seq_cycle += 1;
            if self.frame_seq_cycle >= 8192 {
                self.frame_seq_cycle -= 8192;
                self.tick_frame_sequencer();
            }

            self.sample_counter += SAMPLE_RATE;
            if self.sample_counter >= CPU_CLOCK {
                self.sample_counter -= CPU_CLOCK;
                self.generate_sample(io_regs);
            }
        }

        self.update_nr52(io_regs);
    }
}

fn tick_length_sq(ch: &mut SquareChannel) {
    if ch.length_enable && ch.length_counter > 0 {
        ch.length_counter -= 1;
        if ch.length_counter == 0 { ch.enabled = false; }
    }
}

fn tick_length_wave(ch: &mut WaveChannel) {
    if ch.length_enable && ch.length_counter > 0 {
        ch.length_counter -= 1;
        if ch.length_counter == 0 { ch.enabled = false; }
    }
}

fn tick_length_noise(ch: &mut NoiseChannel) {
    if ch.length_enable && ch.length_counter > 0 {
        ch.length_counter -= 1;
        if ch.length_counter == 0 { ch.enabled = false; }
    }
}

fn tick_env_sq(ch: &mut SquareChannel) {
    if ch.envelope_period > 0 && ch.dac_enabled {
        ch.envelope_counter -= 1;
        if ch.envelope_counter <= 0 {
            ch.envelope_counter = ch.envelope_period as i32;
            if ch.envelope_dir != 0 {
                if ch.volume < 15 { ch.volume += 1; }
            } else {
                if ch.volume > 0 { ch.volume -= 1; }
            }
        }
    }
}

fn tick_env_noise(ch: &mut NoiseChannel) {
    if ch.envelope_period > 0 && ch.dac_enabled {
        ch.envelope_counter -= 1;
        if ch.envelope_counter <= 0 {
            ch.envelope_counter = ch.envelope_period as i32;
            if ch.envelope_dir != 0 {
                if ch.volume < 15 { ch.volume += 1; }
            } else {
                if ch.volume > 0 { ch.volume -= 1; }
            }
        }
    }
}
