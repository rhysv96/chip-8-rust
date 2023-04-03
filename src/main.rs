extern crate sdl2;

use std::thread;
use std::fs;
use std::fs::File;
use std::io::Read;

use ticktock::Clock;

pub mod vm;
pub mod screen;

const RENDER_TICKSPEED: f64 = 60.0;
const CPU_TICKSPEED: f64 = 700.0;

pub fn main() {
    let mut sys = vm::System::new();
    let mut screen = screen::Screen::new();
    let rom_data = get_file_as_byte_vec(&String::from("./IBM Logo.ch8"));
    sys.load_rom(rom_data);
    // temp slow tick
    // TODO: fix this
    for (_tick, _now) in Clock::framerate(RENDER_TICKSPEED).iter() {
        // hack, tick 23 times and then draw
        for _ in 0..23 as i32 {
            sys.tick();
        }
        screen.draw_screen(&sys.screen);
    }
    /*
    // render tick
    thread::spawn(|| {
        for (tick, now) in Clock::framerate(RENDER_TICKSPEED).iter() {
            println!("render");
        }
    });

    // cpu tick
    for (tick, now) in Clock::framerate(CPU_TICKSPEED).iter() {
        sys.tick();
    }
    */
}

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}