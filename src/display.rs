use sdl2::pixels::Color;
use sdl2::video::Window;
use sdl2::{render::Canvas, Sdl};

const WIDTH: u32 = 64; // Pixels
const HEIGHT: u32 = 32; // Pixels

const WIDTH_PER_PIXEL: u32 = 20;
const HEIGHT_PER_PIXEL: u32 = 20;

const PIXEL_OFF_COLOR: Color = Color::RGB(0x99, 0x66, 0x01);
//const PIXEL_OFF_COLOR: Color = Color::RGB(0x0, 0x0, 0x0);
const PIXEL_ON_COLOR: Color = Color::RGB(0xff, 0xcc, 0x01);

pub struct Display {
    pub sdl_context: Sdl,
    pub canvas: Canvas<Window>,
}

impl Display {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("Chip-8", WIDTH * WIDTH_PER_PIXEL, HEIGHT * HEIGHT_PER_PIXEL)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();
        let canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut screen = Display {
            sdl_context: sdl_context,
            canvas: canvas,
        };

        screen.clear();
        screen.canvas.clear();
        screen.canvas.present();

        screen
    }

    pub fn to_on_color(&mut self) {
        self.canvas.set_draw_color(PIXEL_ON_COLOR);
    }

    pub fn to_off_color(&mut self) {
        self.canvas.set_draw_color(PIXEL_OFF_COLOR);
    }

    pub fn clear(&mut self) {
        self.to_off_color();
        self.canvas.clear();
    }

    pub fn get_height_per_pixel() -> u32 {
        HEIGHT_PER_PIXEL
    }

    pub fn get_width_per_pixel() -> u32 {
        WIDTH_PER_PIXEL
    }
}
