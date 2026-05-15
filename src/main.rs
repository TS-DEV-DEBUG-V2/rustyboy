mod types;
mod bios;
mod cartridge;
mod input;
mod apu;
mod memory;
mod cpu;
mod ppu;
mod timer;

use types::*;
use cpu::Cpu;
use memory::Memory;
use ppu::Ppu;
use timer::Timer;

use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::EventPump;

use std::path::PathBuf;
use std::time::Instant;

const TARGET_FPS: u64 = 60;
const FRAME_MS: u64 = 1000 / TARGET_FPS;
const SAVE_INTERVAL: i32 = 300;

struct GbState {
    cpu: Cpu,
    mem: Memory,
    ppu: Ppu,
    timer: Timer,

    save_path: String,
    #[allow(dead_code)]
    save_key: String,
    save_timer: i32,

    running: bool,
    frame_start: Instant,
}

struct SdlState {
    canvas: Canvas<Window>,
    texture: Texture<'static>,
    audio_queue: Option<AudioQueue<i16>>,
    event_pump: EventPump,
}

fn sdl_to_gb(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Right  => Some(input::BTN_RIGHT),
        Keycode::Left   => Some(input::BTN_LEFT),
        Keycode::Up     => Some(input::BTN_UP),
        Keycode::Down   => Some(input::BTN_DOWN),
        Keycode::Z      => Some(input::BTN_A),
        Keycode::X      => Some(input::BTN_B),
        Keycode::Return => Some(input::BTN_START),
        Keycode::RShift => Some(input::BTN_SELECT),
        Keycode::LShift => Some(input::BTN_SELECT),
        _ => None,
    }
}

fn handle_event(e: Event, gb: &mut GbState, sdl: &mut SdlState) {
    match e {
        Event::Quit { .. } => { gb.running = false; }
        Event::Window { win_event: WindowEvent::Resized(w, h), .. } => {
            let fit_w = h * (SCREEN_W as i32) / (SCREEN_H as i32);
            let fit_h = w * (SCREEN_H as i32) / (SCREEN_W as i32);
            let viewport = if fit_w <= w {
                let vw = fit_w;
                let vh = h;
                let vx = (w - vw) / 2;
                let vy = 0;
                Rect::new(vx, vy, vw as u32, vh as u32)
            } else {
                let vw = w;
                let vh = fit_h;
                let vx = 0;
                let vy = (h - vh) / 2;
                Rect::new(vx, vy, vw as u32, vh as u32)
            };
            sdl.canvas.set_viewport(viewport);
        }
        Event::KeyDown { keycode: Some(kc), .. } => {
            if kc == Keycode::Escape { gb.running = false; }
            if let Some(b) = sdl_to_gb(kc) {
                gb.mem.input.key_down(b);
                gb.mem.io_regs[0x0F] |= INT_JOYPAD;
            }
        }
        Event::KeyUp { keycode: Some(kc), .. } => {
            if let Some(b) = sdl_to_gb(kc) {
                gb.mem.input.key_up(b);
            }
        }
        _ => {}
    }
}

fn run_frame(gb: &mut GbState, sdl: &mut SdlState) {
    let target_cycles: u64 = 70224;
    let mut total: u64 = 0;

    while total < target_cycles {
        let cycles = gb.cpu.tick(&mut gb.mem);
        gb.ppu.step(cycles, &mut gb.mem);
        gb.timer.step(cycles, &mut gb.mem);
        // split borrow: apu and io_regs both live inside mem
        let mem_ref = &mut gb.mem;
        // SAFETY: apu.step only mutates the io_regs slice we pass it (which is disjoint from apu).
        let io_ptr: *mut [u8; 0x80] = &mut mem_ref.io_regs;
        unsafe {
            mem_ref.apu.step(cycles, &mut *io_ptr);
        }
        total += cycles as u64;
    }

    let pitch = SCREEN_W * 4;
    let bytes: &[u8] = unsafe {
        std::slice::from_raw_parts(
            gb.ppu.pixels.as_ptr() as *const u8,
            gb.ppu.pixels.len() * 4,
        )
    };
    let _ = sdl.texture.update(None, bytes, pitch);
    sdl.canvas.clear();
    let _ = sdl.canvas.copy(&sdl.texture, None, None);
    sdl.canvas.present();

    if let Some(ref aq) = sdl.audio_queue {
        if gb.mem.apu.buffer_pos > 0 {
            let _ = aq.queue_audio(&gb.mem.apu.audio_buffer[..gb.mem.apu.buffer_pos]);
            gb.mem.apu.buffer_pos = 0;
        }
    }

    gb.save_timer += 1;
    if gb.mem.cart.dirty && gb.save_timer >= SAVE_INTERVAL {
        gb.save_timer = 0;
        gb.mem.cart.dirty = false;
        save_ram_to_file(gb);
    }

    let elapsed = gb.frame_start.elapsed().as_millis() as u64;
    if elapsed < FRAME_MS {
        let remaining = FRAME_MS - elapsed;
        std::thread::sleep(std::time::Duration::from_millis(remaining));
    }
    gb.frame_start = Instant::now();
}

fn save_ram_to_file(gb: &mut GbState) -> bool {
    if gb.mem.cart.ram_size() == 0 { return false; }
    gb.mem.cart.save_ram(&gb.save_path);
    persist_save_js(gb);
    true
}

fn load_ram_from_file(gb: &mut GbState) -> bool {
    if gb.mem.cart.ram_size() == 0 { return false; }
    gb.mem.cart.load_ram(&gb.save_path)
}

#[cfg(target_os = "emscripten")]
fn persist_save_js(gb: &GbState) {
    use std::ffi::CString;
    if gb.save_key.is_empty() { return; }
    extern "C" {
        fn emscripten_run_script(s: *const std::os::raw::c_char);
    }
    let script = format!(
        "try {{ var data = FS.readFile('{}'); var bin = ''; for (var i = 0; i < data.length; i++) bin += String.fromCharCode(data[i]); localStorage.setItem('{}', btoa(bin)); }} catch(e) {{}}",
        gb.save_path.replace('\'', "\\'"),
        gb.save_key.replace('\'', "\\'")
    );
    let cs = CString::new(script).unwrap();
    unsafe { emscripten_run_script(cs.as_ptr()); }
}

#[cfg(not(target_os = "emscripten"))]
fn persist_save_js(_gb: &GbState) {}

#[cfg(target_os = "emscripten")]
static mut GB_STATE: Option<GbState> = None;
#[cfg(target_os = "emscripten")]
static mut SDL_STATE: Option<SdlState> = None;

#[cfg(target_os = "emscripten")]
extern "C" {
    fn emscripten_set_main_loop(
        func: unsafe extern "C" fn(),
        fps: std::os::raw::c_int,
        simulate_infinite_loop: std::os::raw::c_int,
    );
}

#[cfg(target_os = "emscripten")]
unsafe extern "C" fn emscripten_loop_fn() {
    let gb = match GB_STATE.as_mut() { Some(g) => g, None => return };
    let sdl = match SDL_STATE.as_mut() { Some(s) => s, None => return };

    let events: Vec<Event> = sdl.event_pump.poll_iter().collect();
    for e in events {
        handle_event(e, gb, sdl);
    }
    run_frame(gb, sdl);
}

fn init_sdl() -> Result<SdlState, String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let audio_sub = sdl.audio()?;
    let event_pump = sdl.event_pump()?;

    // Leak the sdl context so subsystems remain valid for the lifetime of the program.
    // (Required because we leak the TextureCreator below; resources may outlive locals.)
    let _ = Box::leak(Box::new(sdl));
    let _ = Box::leak(Box::new(video.clone()));
    let _ = Box::leak(Box::new(audio_sub.clone()));

    let mut window_builder = video.window(
        "rustyboy by TS copyright 2026 ",
        (SCREEN_W as u32) * 3,
        (SCREEN_H as u32) * 3,
    );
    window_builder.position_centered();

    #[cfg(not(target_os = "emscripten"))]
    {
        window_builder.resizable().allow_highdpi();
    }

    let window = window_builder.build().map_err(|e| e.to_string())?;

    let mut canvas_builder = window.into_canvas().accelerated();
    #[cfg(not(target_os = "emscripten"))]
    {
        canvas_builder = canvas_builder.present_vsync();
    }
    let canvas = canvas_builder.build().map_err(|e| e.to_string())?;

    // Leak the TextureCreator to obtain a 'static reference, so Texture<'static> is sound.
    let tc_boxed: Box<TextureCreator<WindowContext>> = Box::new(canvas.texture_creator());
    let tc_static: &'static TextureCreator<WindowContext> = Box::leak(tc_boxed);

    let texture = tc_static
        .create_texture(
            PixelFormatEnum::ABGR8888,
            sdl2::render::TextureAccess::Streaming,
            SCREEN_W as u32,
            SCREEN_H as u32,
        )
        .map_err(|e| e.to_string())?;

    let audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(2048),
    };
    let audio_queue: Option<AudioQueue<i16>> = match audio_sub.open_queue::<i16, _>(None, &audio_spec) {
        Ok(q) => { q.resume(); Some(q) }
        Err(e) => { eprintln!("audio init failed: {}", e); None }
    };

    Ok(SdlState {
        canvas,
        texture,
        audio_queue,
        event_pump,
    })
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rustyboy <rom.gb>");
        return Err("missing arg".to_string());
    }

    let mut gb = GbState {
        cpu: Cpu::new(),
        mem: Memory::new(),
        ppu: Ppu::new(),
        timer: Timer::new(),
        save_path: String::new(),
        save_key: String::new(),
        save_timer: 0,
        running: true,
        frame_start: Instant::now(),
    };

    if !gb.mem.cart.load(&args[1]) {
        eprintln!("Failed to load ROM: {}", args[1]);
        return Err("rom load failed".to_string());
    }

    gb.mem.load_boot_rom(&bios::DMG_BOOT);

    #[cfg(target_os = "emscripten")]
    {
        gb.save_path = if args.len() > 2 { args[2].clone() } else { "save.sav".to_string() };
        gb.save_key = if args.len() > 3 { args[3].clone() } else { "gb_save_data".to_string() };
        if gb.mem.cart.ram_size() > 0 { load_ram_from_file(&mut gb); }
    }
    #[cfg(not(target_os = "emscripten"))]
    {
        let mut p = PathBuf::from(&args[1]);
        let new_name = format!("{}.sav", p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default());
        p.set_file_name(new_name);
        gb.save_path = p.to_string_lossy().to_string();
        if std::path::Path::new(&gb.save_path).exists() {
            load_ram_from_file(&mut gb);
        }
    }

    let mut sdl = init_sdl()?;

    gb.cpu.reset();
    gb.mem.reset();
    gb.ppu.reset();
    gb.timer.reset();
    gb.mem.apu.reset();

    println!(
        "Loaded: {} | Type: {:?} | ROM Banks: {} | RAM Banks: {}",
        args[1], gb.mem.cart.mbc_type, gb.mem.cart.num_rom_banks, gb.mem.cart.num_ram_banks
    );

    gb.frame_start = Instant::now();

    #[cfg(target_os = "emscripten")]
    unsafe {
        GB_STATE = Some(gb);
        SDL_STATE = Some(sdl);
        emscripten_set_main_loop(emscripten_loop_fn, 0, 1);
    }

    #[cfg(not(target_os = "emscripten"))]
    {
        while gb.running {
            let events: Vec<Event> = sdl.event_pump.poll_iter().collect();
            for e in events {
                handle_event(e, &mut gb, &mut sdl);
            }
            run_frame(&mut gb, &mut sdl);
        }
        save_ram_to_file(&mut gb);
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}
