extern crate sdl2;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use sdl2::event::Event;
use sdl2::keyboard::Scancode;

pub mod vm;
pub mod screen;

pub fn main() {
    let mut sys = vm::System::new();
    let rom_data = get_file_as_byte_vec(&String::from("./test_opcode.ch8"));
    sys.load_rom(rom_data);

    let mut sys = Arc::new(RwLock::new(sys));

    let sys_tick = sys.clone();
    let ticker_thread = thread::spawn(move || {
        let ticks_per_second = 700;
        let target_interval = Duration::from_micros(1_000_000 / ticks_per_second);

        let mut local_sys = sys_tick.read().unwrap().clone();

        loop {
            let tick_start = Instant::now();

            let write_start = Instant::now();
            {
                let sys = sys_tick.read().unwrap();
                if let vm::Status::Terminated = sys.status {
                    break;
                }
                sys.tick(&mut local_sys);
            }
            let write_elapsed = write_start.elapsed();

            let clone_start = Instant::now();
            {
                let sys_clone = local_sys.clone();
                let mut sys = sys_tick.write().unwrap();
                *sys = sys_clone;
            }
            let clone_elapsed = clone_start.elapsed();

            let elapsed = tick_start.elapsed();
            if elapsed < target_interval {
                thread::sleep(target_interval - elapsed);
            } else if elapsed > target_interval * 10 {
                println!(
                    "ticker ticked for far too long! {}μs, should be {}μs ({}μs ticking, {}μs cloning)",
                    elapsed.as_micros(),
                    target_interval.as_micros(),
                    write_elapsed.as_micros(),
                    clone_elapsed.as_micros(),
                );
            }


            /*
            if sys.sound_timer > 0 {
                // TODO: beep
                println!("beep!");
            }
            */
        }
    });


    let sys_render = sys.clone();
    let render_thread = thread::spawn(move || {
        let ticks_per_second = 60;
        let target_interval = Duration::from_micros(1_000_000 / ticks_per_second);

        let mut screen = screen::Screen::new();

        'main: loop {
            let tick_start = Instant::now();

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

            {
                if sys_render.read().unwrap().keys != input {
                    let mut sys = sys_render.write().unwrap();
                    sys.keys = input;
                }
            }

            'event: loop {
                match screen.event_pump.poll_event() {
                    Some(event) => match event {
                        Event::Quit { timestamp: _ } => {
                            let mut sys = sys_render.write().unwrap();
                            sys.status = vm::Status::Terminated
                        }
                        _ => {},
                    }
                    None => break 'event,
                }
            }

            {
                let sys = sys_render.read().unwrap();
                screen.draw_screen(&sys.screen);


                if let vm::Status::Terminated = sys.status {
                    break 'main;
                }
            }


            let elapsed = tick_start.elapsed();
            if elapsed < target_interval {
                thread::sleep(target_interval - elapsed);
            }
        }
    });

    ticker_thread.join().unwrap();
    render_thread.join().unwrap();
}

fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
    let mut f = File::open(&filename).expect("no file found");
    let metadata = fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}