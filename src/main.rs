extern crate sdl2;

use system::System;

pub mod vm;
pub mod io;
pub mod system;

pub fn main() {
    let mut sys = System::new();
    sys.init(String::from("pong.ch8"));
}