use sdl2::{event::Event, keyboard::Keycode, EventPump};

use crate::display::Display;

#[derive(PartialEq)]
pub enum InputAction {
    BreakDisplay,
    None,
}

pub struct Input {
    pub key_pad: [bool; 16],
    event_pump: EventPump,
}

impl Input {
    pub fn new(display: &mut Display) -> Self {
        let event_pump = display.sdl_context.event_pump().unwrap();

        Self {
            event_pump: event_pump,
            key_pad: [true; 16],
        }
    }

    pub fn handle_input(&mut self) -> InputAction {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => return InputAction::BreakDisplay,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return InputAction::BreakDisplay,
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => self.key_pad[0] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => self.key_pad[1] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => self.key_pad[2] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => self.key_pad[3] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => self.key_pad[4] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => self.key_pad[5] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => self.key_pad[6] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => self.key_pad[7] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => self.key_pad[8] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => self.key_pad[9] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => self.key_pad[10] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => self.key_pad[11] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => self.key_pad[12] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => self.key_pad[13] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => self.key_pad[14] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => self.key_pad[15] = true,
                _ => {}
            }
        }

        InputAction::None
    }
}
