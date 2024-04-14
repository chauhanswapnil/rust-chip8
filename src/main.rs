use crate::chip8::{Chip8, SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::env;
use std::fs::File;
use std::io::Read;

mod chip8;

// The original display is 64 x 32. Scale it according to our needs.
const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 15;

fn main() {
    println!("Welcome to Chip8...");

    // TODO: Take SCALE and TICKS_PER_FRAME as user input.
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        eprintln!("Usage: chip8 path/to/rom");
        return;
    }
    let rom_file_path = &args[1];
    let mut rom = File::open(rom_file_path).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();

    // Setup SDL2 (Taken from https://docs.rs/sdl2/latest/sdl2/#functions)
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Rust Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Chip8::new();
    chip8.load(&buffer);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                // Keydown is registered as a keypress held
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_button(key) {
                        chip8.keypress(k, true);
                    }
                }
                // Keyup is registered as keypress lifted
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = key_to_button(key) {
                        chip8.keypress(k, false);
                    }
                }
                _ => (),
            }
        }
        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }
        chip8.tick_timers();
        // Draw the screen black
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let screen = chip8.get_screen();
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (i, pixel) in screen.iter().enumerate() {
            if *pixel {
                let x = (i % SCREEN_WIDTH) as u32;
                let y = (i / SCREEN_WIDTH) as u32;

                let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
                canvas.fill_rect(rect).unwrap();
            }
        }
        canvas.present();
    }
}

fn key_to_button(key: Keycode) -> Option<usize> {
    /*
    COSMAC VIP used the following layout, which was then re-used on the HP48 calculators,
    This is the standard keypad used in emulators.
    We map them with the left hand side of the keyboard.
        1	2	3	C
        4	5	6	D
        7	8	9	E
        A	0	B	F
    */

    // TODO: Change it so its not dependent on English Keyboard
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
