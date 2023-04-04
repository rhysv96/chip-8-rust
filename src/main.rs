extern crate sdl2;

use std::fs;
use std::fs::File;
use std::io::Read;

use sdl2::event::Event;

pub mod vm;
pub mod screen;

pub fn main() {
    let mut sys = vm::System::new();
    let mut screen = screen::Screen::new();
    let rom_data = get_file_as_byte_vec(&String::from("./test_opcode.ch8"));
    sys.load_rom(rom_data);

    'main: loop {
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