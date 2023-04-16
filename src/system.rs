use std::fs::{File, self};
use std::io::Read;
use std::sync::{Arc, RwLock};
use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use std::thread::{JoinHandle, self};
use std::time::{Duration, Instant};

use rodio::source::SineWave;
use rodio::{OutputStream, Sink, Source};
use sdl2::event::Event;
use sdl2::keyboard::Scancode;

use super::vm::{ VM, Screen, Status };
use super::io::IO;

enum Signal {
    DecrementDelayTimer,
    DecrementSoundTimer,
    Terminate,
    SendKeys(u16),
}

pub struct Interfaces {
    pub screen: Screen,
    pub sound_timer: u8,
    pub keys: u16,
}

impl Interfaces {
    pub fn new() -> Self {
        Interfaces {
            screen: VM::create_screen(),
            sound_timer: 0,
            keys: 0,
        }
    }
}

impl Clone for Interfaces {
    fn clone(&self) -> Self {
        Self { screen: self.screen.clone(), sound_timer: self.sound_timer.clone(), keys: self.keys.clone() }
    }
}

pub struct System {
    interfaces: Arc<RwLock<Interfaces>>,
}

impl System {
    pub fn new() -> Self {
        Self {
            interfaces: Arc::new(RwLock::new(Interfaces::new())),
        }
    }

    pub fn init(&mut self, rom_path: String) {
        let (vm_thread, sender) = self.start_vm_thread(rom_path);
        let io_thread = self.start_io_thread(sender);

        vm_thread.join().expect("vm thread panicked");
        io_thread.join().expect("io thread panicked");
    }

    fn get_file_as_byte_vec(filename: String) -> Vec<u8> {
        let mut f = File::open(&filename).expect("no file found");
        let metadata = fs::metadata(&filename).expect("unable to read metadata");
        if metadata.len() > 4096 {
            panic!("filesize too large");
        }
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");

        buffer
    }

    fn start_vm_thread(&mut self, rom_path: String) -> (JoinHandle<()>, Sender<Signal>) {
        let (sender, receiver): (Sender<Signal>, Receiver<Signal>) = channel();
        let interfaces = self.interfaces.clone();
        let vm_thread = thread::spawn(move || {
            println!("Starting vm thread");
            let ticks_per_second = 700;
            let target_interval = Duration::from_micros(1_000_000 / ticks_per_second);

            let mut vm = VM::new();
            vm.load_rom(Self::get_file_as_byte_vec(rom_path));
            let mut local_interfaces = Interfaces::new();

            loop {
                let tick_start = Instant::now();

                let write_start = Instant::now();
                {
                    'delay: loop {
                        match receiver.try_recv() {
                            Ok(signal) => match signal {
                                Signal::DecrementDelayTimer => if vm.delay_timer > 0 { vm.delay_timer -= 1 }
                                Signal::DecrementSoundTimer => if local_interfaces.sound_timer > 0 { local_interfaces.sound_timer -= 1 }
                                Signal::Terminate => vm.terminate(),
                                Signal::SendKeys(keys) => local_interfaces.keys = keys,
                            },
                            Err(TryRecvError::Empty) => {
                                break 'delay;
                            },
                            Err(TryRecvError::Disconnected) => {
                                panic!("The signal channel has been disconnected");
                            },
                        }
                    }

                    if let Status::Terminated = vm.status {
                        break;
                    }

                    vm.tick(&mut local_interfaces);
                }
                let write_elapsed = write_start.elapsed();

                let clone_start = Instant::now();
                {
                    let new_interfaces = local_interfaces.clone();
                    let mut interfaces = interfaces.write().unwrap();
                    *interfaces = new_interfaces;
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


            }
        });
        (vm_thread, sender)
    }

    fn start_io_thread(&mut self, sender: Sender<Signal>) -> JoinHandle<()> {
        let interfaces = self.interfaces.clone();
        let io_thread = thread::spawn(move || {
            println!("Starting io thread");

            let ticks_per_second = 60;
            let target_interval = Duration::from_micros(1_000_000 / ticks_per_second);

            let mut io = IO::new();

            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();
            let mut beeping = false;

            'main: loop {
                let tick_start = Instant::now();

                let keeb = io.event_pump.keyboard_state();
                let mut input: u16 = 0;
                // 1 2 3 C
                // 4 5 6 D
                // 7 8 9 E
                // A 0 B F
                if keeb.is_scancode_pressed(Scancode::Num1) { input += 1 << 1; } // 1
                if keeb.is_scancode_pressed(Scancode::Num2) { input += 1 << 2; } // 2
                if keeb.is_scancode_pressed(Scancode::Num3) { input += 1 << 3; } // 3
                if keeb.is_scancode_pressed(Scancode::Num4) { input += 1 << 12; } // C
                if keeb.is_scancode_pressed(Scancode::Q) { input += 1 << 4; } // 4
                if keeb.is_scancode_pressed(Scancode::W) { input += 1 << 5; } // 5
                if keeb.is_scancode_pressed(Scancode::E) { input += 1 << 6; } // 6
                if keeb.is_scancode_pressed(Scancode::R) { input += 1 << 13; } // D
                if keeb.is_scancode_pressed(Scancode::A) { input += 1 << 7; } // 7
                if keeb.is_scancode_pressed(Scancode::S) { input += 1 << 8; } // 8
                if keeb.is_scancode_pressed(Scancode::D) { input += 1 << 9; } // 9
                if keeb.is_scancode_pressed(Scancode::F) { input += 1 << 14; } // E
                if keeb.is_scancode_pressed(Scancode::Z) { input += 1 << 10; } // A
                if keeb.is_scancode_pressed(Scancode::X) { input += 1 } // 0
                if keeb.is_scancode_pressed(Scancode::C) { input += 1 << 11; } // B
                if keeb.is_scancode_pressed(Scancode::V) { input += 1 << 15; } // F

                sender.send(Signal::SendKeys(input)).unwrap();

                if keeb.is_scancode_pressed(Scancode::Escape) {
                    sender.send(Signal::Terminate).unwrap();
                    break 'main;
                }

                'event: loop {
                    match io.event_pump.poll_event() {
                        Some(event) => match event {
                            Event::Quit { timestamp: _ } => {
                                sender.send(Signal::Terminate).unwrap();
                                break 'main;
                            }
                            _ => {},
                        }
                        None => break 'event,
                    }
                }

                {
                    let interfaces = interfaces.read().unwrap();
                    io.draw_screen(&interfaces.screen);
                }

                // ask the ticker thread to decrement delay timers
                sender.send(Signal::DecrementDelayTimer).unwrap();
                sender.send(Signal::DecrementSoundTimer).unwrap();

                {
                    let sound_timer = { interfaces.read().unwrap().sound_timer };
                    if sound_timer > 0 && !beeping {
                        let source = SineWave::new(500.0).take_duration(Duration::from_secs(5)).amplify(0.2);
                        sink.append(source);
                        beeping = true;
                    } else if sound_timer == 0 && beeping {
                        sink.clear();
                        beeping = false;
                    }
                }

                let elapsed = tick_start.elapsed();
                if elapsed < target_interval {
                    thread::sleep(target_interval - elapsed);
                }
            }
        });

        io_thread
    }
}