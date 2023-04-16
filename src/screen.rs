extern crate sdl2;

use crate::vm::{SCREEN_WIDTH, SCREEN_HEIGHT};

use super::vm;

use sdl2::{ EventPump };
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

const PIXEL_SCALING: u32 = 30;
const RENDER_GRID: bool = false;

pub struct Screen {
    canvas: Canvas<Window>,
    pub event_pump: EventPump,
}

impl Screen {
    pub fn new() -> Screen {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "chip-8-rs",
                vm::SCREEN_WIDTH as u32 * PIXEL_SCALING,
                vm::SCREEN_HEIGHT as u32 * PIXEL_SCALING
            )
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        let event_pump = sdl_context.event_pump().unwrap();

        Screen {
            canvas,
            event_pump,
        }
    }

    pub fn draw_screen(&mut self, pixels: &[[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]) {
        let c = &mut self.canvas;
        let off = Color::RGB(0, 0, 0);
        let on = Color::RGB(255, 255, 255);
        let scale = PIXEL_SCALING as usize;

        c.set_draw_color(off);
        c.clear();

        // pixels
        for y in 0..SCREEN_HEIGHT {
            for x in 0..SCREEN_WIDTH {
                if pixels[y][x] {
                    c.set_draw_color(on);
                } else {
                    c.set_draw_color(off);
                }
                c.fill_rect(Rect::new((x * scale) as i32, (y * scale) as i32, scale as u32, scale as u32)).unwrap();
            }
        }

        // grid
        if RENDER_GRID {
            for y in 0..SCREEN_HEIGHT {
                for x in 0..SCREEN_WIDTH {
                    c.set_draw_color(Color::RGB(64, 64, 64));
                    c.draw_rect(Rect::new(
                        (x * scale) as i32,
                        (y * scale) as i32,
                        scale as u32,
                        scale as u32,
                    )).unwrap();
                }
            }
        }

        c.present();
    }
}
