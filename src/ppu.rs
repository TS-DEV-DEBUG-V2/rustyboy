use crate::types::*;
use crate::memory::Memory;

const COLORS: [u32; 4] = [0xFFFFFFFF, 0xFFAAAAAA, 0xFF555555, 0xFF000000];

const OAM_CYCLES: i32 = 80;
const MODE3_BASE: i32 = 172;
const MODE3_PER_SPRITE: i32 = 8;
const CYCLES_PER_LINE: i32 = 456;

#[derive(Default, Clone, Copy)]
pub struct SpriteInfo {
    pub x: u8,
    pub tile: u8,
    pub flags: u8,
    pub y: i32,
    pub index: u8,
}

pub struct Ppu {
    pub framebuffer: Vec<u32>,
    pub pixels: Vec<u32>,

    pub dot_counter: u32,
    pub mode: PpuMode,
    pub frame_count: u32,
    pub lcd_was_enabled: bool,
    pub mode3_duration: i32,

    pub visible_sprites: [SpriteInfo; 10],
    pub num_sprites: usize,

    pub bg_tile_cache: [[u8; 8]; 21],
    pub win_tile_cache: [[u8; 8]; 21],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            framebuffer: vec![0xFFFFFFFFu32; SCREEN_W * SCREEN_H],
            pixels: vec![0xFFFFFFFFu32; SCREEN_W * SCREEN_H],
            dot_counter: 0,
            mode: PpuMode::OamScan,
            frame_count: 0,
            lcd_was_enabled: false,
            mode3_duration: 0,
            visible_sprites: [SpriteInfo::default(); 10],
            num_sprites: 0,
            bg_tile_cache: [[0u8; 8]; 21],
            win_tile_cache: [[0u8; 8]; 21],
        }
    }

    pub fn reset(&mut self) {
        for v in self.framebuffer.iter_mut() { *v = 0xFFFFFFFF; }
        for v in self.pixels.iter_mut() { *v = 0xFFFFFFFF; }
        self.dot_counter = 0;
        self.mode = PpuMode::OamScan;
        self.frame_count = 0;
        self.lcd_was_enabled = false;
        self.num_sprites = 0;
        self.mode3_duration = 0;
    }

    fn get_color(palette: u8, color_idx: u8) -> u32 {
        let shift = color_idx * 2;
        let idx = ((palette >> shift) & 3) as usize;
        COLORS[idx]
    }

    fn decode_tile_row(byte1: u8, byte2: u8, out: &mut [u8; 8]) {
        out[0] = (((byte2 >> 7) & 1) << 1) | ((byte1 >> 7) & 1);
        out[1] = (((byte2 >> 6) & 1) << 1) | ((byte1 >> 6) & 1);
        out[2] = (((byte2 >> 5) & 1) << 1) | ((byte1 >> 5) & 1);
        out[3] = (((byte2 >> 4) & 1) << 1) | ((byte1 >> 4) & 1);
        out[4] = (((byte2 >> 3) & 1) << 1) | ((byte1 >> 3) & 1);
        out[5] = (((byte2 >> 2) & 1) << 1) | ((byte1 >> 2) & 1);
        out[6] = (((byte2 >> 1) & 1) << 1) | ((byte1 >> 1) & 1);
        out[7] = ((byte2 & 1) << 1) | (byte1 & 1);
    }

    fn scan_oam(&mut self, mem: &Memory, ly: u8, sprite_h: i32) {
        self.num_sprites = 0;
        for i in 0..40 {
            if self.num_sprites >= 10 { break; }
            let oam_addr = 0xFE00u16 + (i as u16) * 4;
            let sy = mem.read(oam_addr);
            let sx = mem.read(oam_addr + 1);
            let stile = mem.read(oam_addr + 2);
            let sflags = mem.read(oam_addr + 3);
            if sy == 0 || sy >= 160 { continue; }
            let spr_y = (sy as i32) - 16;
            if (ly as i32) >= spr_y && (ly as i32) < spr_y + sprite_h {
                let si = SpriteInfo {
                    x: sx, tile: stile, flags: sflags,
                    y: spr_y, index: i as u8,
                };
                self.visible_sprites[self.num_sprites] = si;
                self.num_sprites += 1;
            }
        }

        // Stable sort by (x, index)
        let n = self.num_sprites;
        let slice = &mut self.visible_sprites[..n];
        slice.sort_by(|a, b| {
            if a.x != b.x { a.x.cmp(&b.x) }
            else { a.index.cmp(&b.index) }
        });
    }

    fn render_line(&mut self, mem: &Memory, ly: u8) {
        let lcdc = mem.read(LCDC);
        let scx = mem.read(SCX);
        let scy = mem.read(SCY);
        let bgp = mem.read(BGP);
        let obp0 = mem.read(OBP0);
        let obp1 = mem.read(OBP1);
        let wy = mem.read(WY);
        let wx = mem.read(WX);

        let bg_enable = (lcdc & 0x01) != 0;
        let obj_enable = (lcdc & 0x02) != 0;
        let obj_size = (lcdc & 0x04) != 0;
        let bg_tile_map = (lcdc & 0x08) != 0;
        let bg_tile_data = (lcdc & 0x10) != 0;
        let win_enable = (lcdc & 0x20) != 0;
        let win_tile_map = (lcdc & 0x40) != 0;
        let lcd_enable = (lcdc & 0x80) != 0;

        if !lcd_enable { return; }

        let bg_map_base: u16 = if bg_tile_map { 0x9C00 } else { 0x9800 };
        let win_map_base: u16 = if win_tile_map { 0x9C00 } else { 0x9800 };
        let tile_data_base: u16 = if bg_tile_data { 0x8000 } else { 0x8800 };
        let signed_tiles = !bg_tile_data;
        let sprite_h: i32 = if obj_size { 16 } else { 8 };

        self.scan_oam(mem, ly, sprite_h);

        let mut bg_pixels = [0u8; SCREEN_W];
        let mut win_pixels = [0u8; SCREEN_W];
        let mut win_active = [false; SCREEN_W];

        let bg_tile_y = ((ly as i32) + (scy as i32)) & 255;
        let bg_tile_row = bg_tile_y / 8;
        let bg_pixel_row = bg_tile_y % 8;

        let win_tile_row = ((ly as i32) - (wy as i32)) / 8;
        let win_pixel_row = ((ly as i32) - (wy as i32)) % 8;
        let mut win_x_offset = (wx as i32) - 7;

        let bg_map_row = bg_map_base + (bg_tile_row as u16) * 32;

        let mut first_tile_x: i32 = 0;

        if bg_enable {
            first_tile_x = ((scx >> 3) & 31) as i32;
            for t in 0..21 {
                let map_col = (first_tile_x + t) & 31;
                let tile_num = mem.read(bg_map_row + (map_col as u16));

                let tile_addr: u16 = if signed_tiles {
                    let signed = (tile_num as i8) as i32 + 128;
                    tile_data_base.wrapping_add((signed as u16).wrapping_mul(16)).wrapping_add((bg_pixel_row as u16) * 2)
                } else {
                    tile_data_base.wrapping_add((tile_num as u16).wrapping_mul(16)).wrapping_add((bg_pixel_row as u16) * 2)
                };

                let byte1 = mem.read(tile_addr);
                let byte2 = mem.read(tile_addr.wrapping_add(1));
                Self::decode_tile_row(byte1, byte2, &mut self.bg_tile_cache[t as usize]);
            }

            for x in 0..SCREEN_W {
                let bg_x = ((x as i32) + (scx as i32)) & 255;
                let tile_x = bg_x >> 3;
                let pixel_x = bg_x & 7;
                let cache_idx = ((tile_x - first_tile_x) & 31) as usize;
                bg_pixels[x] = self.bg_tile_cache[cache_idx][pixel_x as usize];
            }
        }

        if win_enable && ly >= wy && wx <= 166 {
            if win_x_offset < 0 { win_x_offset = 0; }
            let win_map_row = win_map_base + (win_tile_row as u16) * 32;

            for t in 0..21 {
                let map_col = t & 31;
                let tile_num_w = mem.read(win_map_row + (map_col as u16));

                let tile_addr_w: u16 = if signed_tiles {
                    let signed = (tile_num_w as i8) as i32 + 128;
                    tile_data_base.wrapping_add((signed as u16).wrapping_mul(16)).wrapping_add((win_pixel_row as u16) * 2)
                } else {
                    tile_data_base.wrapping_add((tile_num_w as u16).wrapping_mul(16)).wrapping_add((win_pixel_row as u16) * 2)
                };

                let byte1w = mem.read(tile_addr_w);
                let byte2w = mem.read(tile_addr_w.wrapping_add(1));
                Self::decode_tile_row(byte1w, byte2w, &mut self.win_tile_cache[t as usize]);
            }

            for x in 0..SCREEN_W {
                let win_draw_x = (x as i32) - win_x_offset;
                if win_draw_x >= 0 {
                    let tile_x_w = (win_draw_x >> 3) as usize;
                    let pixel_x_w = (win_draw_x & 7) as usize;
                    if tile_x_w < 21 {
                        let ci = self.win_tile_cache[tile_x_w][pixel_x_w];
                        win_active[x] = true;
                        win_pixels[x] = ci;
                    }
                }
            }
        }

        let mut sprite_pixels = [[0u8; 8]; 10];
        for si in 0..self.num_sprites {
            let s = self.visible_sprites[si];
            let mut pixel_y_s = (ly as i32) - s.y;
            let y_flip = (s.flags & 0x40) != 0;
            if y_flip { pixel_y_s = sprite_h - 1 - pixel_y_s; }

            let mut tile_num_s = s.tile;
            if obj_size { tile_num_s &= 0xFE; }

            let tile_addr = 0x8000u16.wrapping_add((tile_num_s as u16).wrapping_mul(16)).wrapping_add((pixel_y_s as u16) * 2);
            let byte1s = mem.read(tile_addr);
            let byte2s = mem.read(tile_addr.wrapping_add(1));
            Self::decode_tile_row(byte1s, byte2s, &mut sprite_pixels[si]);

            let x_flip = (s.flags & 0x20) != 0;
            if x_flip {
                for i in 0..4 {
                    sprite_pixels[si].swap(i, 7 - i);
                }
            }
        }

        'pixel_loop: for x in 0..SCREEN_W {
            let mut color_idx = bg_pixels[x];
            let mut bg_px = bg_pixels[x];

            if win_active[x] {
                color_idx = win_pixels[x];
                bg_px = win_pixels[x];
            }

            let bg_priority_idx = bg_px;

            if obj_enable {
                for si in 0..self.num_sprites {
                    let s = self.visible_sprites[si];
                    let spr_x = (s.x as i32) - 8;
                    if (x as i32) < spr_x || (x as i32) >= spr_x + 8 { continue; }

                    let pixel_x_s = ((x as i32) - spr_x) as usize;
                    let ci = sprite_pixels[si][pixel_x_s];
                    if ci == 0 { continue; }

                    let bg_priority = (s.flags & 0x80) != 0;
                    if bg_priority && bg_priority_idx != 0 { continue; }

                    let obj_palette = if (s.flags & 0x10) != 0 { obp1 } else { obp0 };
                    color_idx = ci;
                    self.framebuffer[(ly as usize) * SCREEN_W + x] = Self::get_color(obj_palette, color_idx);
                    continue 'pixel_loop;
                }
            }

            self.framebuffer[(ly as usize) * SCREEN_W + x] = Self::get_color(bgp, color_idx);
        }

        let _ = first_tile_x;
    }

    fn update_stat(&self, mem: &mut Memory, iflag: &mut u8) {
        let mut stat = mem.read(STAT);
        stat = (stat & 0xFC) | ((self.mode as u8) & 3);

        let ly = mem.read(LY);
        let lyc = mem.read(LYC);
        let lyc_eq = ly == lyc;
        if lyc_eq { stat |= 0x04; } else { stat &= !0x04; }

        let old_stat_if = (mem.io_regs[0x0F] & INT_STAT) != 0;

        let mut stat_interrupt = false;
        if (stat & 0x40) != 0 && self.mode == PpuMode::Vblank { stat_interrupt = true; }
        if (stat & 0x20) != 0 && self.mode == PpuMode::OamScan { stat_interrupt = true; }
        if (stat & 0x10) != 0 && self.mode == PpuMode::Hblank { stat_interrupt = true; }
        if (stat & 0x08) != 0 && lyc_eq { stat_interrupt = true; }

        mem.write(STAT, stat);

        if stat_interrupt && !old_stat_if {
            *iflag |= INT_STAT;
        }
    }

    pub fn dma_transfer(&self, val: u8, mem: &mut Memory) -> bool {
        let src = (val as u16) << 8;
        for i in 0..0xA0u16 {
            let v = mem.read(src + i);
            mem.write(0xFE00 + i, v);
        }
        true
    }

    pub fn step(&mut self, cycles: u8, mem: &mut Memory) {
        let lcdc = mem.read(LCDC);
        let lcd_ena = (lcdc & 0x80) != 0;

        if !lcd_ena {
            if self.lcd_was_enabled {
                self.lcd_was_enabled = false;
                mem.write(LY, 0);
                self.mode = PpuMode::Hblank;
                self.dot_counter = 0;
                self.num_sprites = 0;
                for v in self.framebuffer.iter_mut() { *v = 0xFFFFFFFF; }
            }
            return;
        }

        if !self.lcd_was_enabled {
            self.lcd_was_enabled = true;
            self.mode = PpuMode::OamScan;
            self.dot_counter = 0;
            mem.write(LY, 0);
        }

        self.dot_counter += cycles as u32;

        loop {
            // Read iflag from io_regs through a local copy so we can write it back
            let mut iflag = mem.io_regs[0x0F];

            match self.mode {
                PpuMode::OamScan => {
                    if (self.dot_counter as i32) < OAM_CYCLES { return; }
                    self.dot_counter -= OAM_CYCLES as u32;
                    self.mode = PpuMode::Draw;
                    let ly = mem.read(LY);
                    self.render_line(mem, ly);
                    let ns = if self.num_sprites > 10 { 10 } else { self.num_sprites as i32 };
                    self.mode3_duration = MODE3_BASE + ns * MODE3_PER_SPRITE;
                    if self.mode3_duration > 289 { self.mode3_duration = 289; }
                    self.update_stat(mem, &mut iflag);
                    mem.io_regs[0x0F] = iflag;
                    continue;
                }
                PpuMode::Draw => {
                    if (self.dot_counter as i32) < self.mode3_duration { return; }
                    self.dot_counter -= self.mode3_duration as u32;
                    self.mode = PpuMode::Hblank;
                    self.update_stat(mem, &mut iflag);
                    mem.io_regs[0x0F] = iflag;
                    continue;
                }
                PpuMode::Hblank => {
                    let hblank_dur = CYCLES_PER_LINE - OAM_CYCLES - self.mode3_duration;
                    if (self.dot_counter as i32) < hblank_dur { return; }
                    self.dot_counter -= hblank_dur as u32;
                    let ly = mem.read(LY).wrapping_add(1);
                    mem.write(LY, ly);
                    if (ly as usize) >= SCREEN_H {
                        self.mode = PpuMode::Vblank;
                        iflag |= INT_VBLANK;
                        self.frame_count += 1;
                        self.pixels.copy_from_slice(&self.framebuffer);
                    } else {
                        self.mode = PpuMode::OamScan;
                    }
                    self.update_stat(mem, &mut iflag);
                    mem.io_regs[0x0F] = iflag;
                    continue;
                }
                PpuMode::Vblank => {
                    if (self.dot_counter as i32) < CYCLES_PER_LINE { return; }
                    self.dot_counter -= CYCLES_PER_LINE as u32;
                    let ly = mem.read(LY).wrapping_add(1);
                    if ly > 153 {
                        mem.write(LY, 0);
                        self.mode = PpuMode::OamScan;
                    } else {
                        mem.write(LY, ly);
                    }
                    self.update_stat(mem, &mut iflag);
                    mem.io_regs[0x0F] = iflag;
                    continue;
                }
            }
        }
    }
}
