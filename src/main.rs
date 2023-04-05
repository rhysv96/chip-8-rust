extern crate sdl2;

use std::fs;
use std::fs::File;
use std::io::Read;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};

pub mod vm;
pub mod screen;

pub fn main() {
    let mut sys = vm::System::new();
    let mut screen = screen::Screen::new();
    let rom_data = get_file_as_byte_vec(&String::from("./test_opcode.ch8"));
    sys.load_rom(rom_data);

    'main: loop {
        let keeb = screen.event_pump.keyboard_state();
        let mut input: u16 = 0;
        if keeb.is_scancode_pressed(Scancode::Num1) { input += 1; }
        if keeb.is_scancode_pressed(Scancode::Num2) { input += 1 << 1; }
        if keeb.is_scancode_pressed(Scancode::Num3) { input += 1 << 2; }
        if keeb.is_scancode_pressed(Scancode::Num4) { input += 1 << 3; }
        if keeb.is_scancode_pressed(Scancode::Q) { input += 1 << 4; }
        if keeb.is_scancode_pressed(Scancode::W) { input += 1 << 5; }
        if keeb.is_scancode_pressed(Scancode::E) { input += 1 << 6; }
        if keeb.is_scancode_pressed(Scancode::R) { input += 1 << 7; }
        if keeb.is_scancode_pressed(Scancode::A) { input += 1 << 8; }
        if keeb.is_scancode_pressed(Scancode::S) { input += 1 << 9; }
        if keeb.is_scancode_pressed(Scancode::D) { input += 1 << 10; }
        if keeb.is_scancode_pressed(Scancode::F) { input += 1 << 11; }
        if keeb.is_scancode_pressed(Scancode::Z) { input += 1 << 12; }
        if keeb.is_scancode_pressed(Scancode::X) { input += 1 << 13; }
        if keeb.is_scancode_pressed(Scancode::C) { input += 1 << 14; }
        if keeb.is_scancode_pressed(Scancode::V) { input += 1 << 15; }
        sys.keys = input;

        sys.tick();

        if sys.drew_in_last_tick {
            screen.draw_screen(&sys.screen);
        }

        'event: loop {
            match screen.event_pump.poll_event() {
                Some(event) => match event {
                    Event::Quit { timestamp: _ } => break 'main,
                    _ => {},
                }
                None => break 'event,
            }
        }
    }
}

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}