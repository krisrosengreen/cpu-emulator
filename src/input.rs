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
            key_pad: [false; 16],
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
                    keycode: Some(Keycode::Num0),
                    ..
                } => self.key_pad[0] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num0),
                    ..
                } => self.key_pad[0] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => self.key_pad[1] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num1),
                    ..
                } => self.key_pad[1] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => self.key_pad[2] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num2),
                    ..
                } => self.key_pad[2] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => self.key_pad[3] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num3),
                    ..
                } => self.key_pad[3] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => self.key_pad[4] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num4),
                    ..
                } => self.key_pad[4] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::Num5),
                    ..
                } => self.key_pad[5] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num5),
                    ..
                } => self.key_pad[5] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::Num6),
                    ..
                } => self.key_pad[6] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num6),
                    ..
                } => self.key_pad[6] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::Num7),
                    ..
                } => self.key_pad[7] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num7),
                    ..
                } => self.key_pad[7] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::Num8),
                    ..
                } => self.key_pad[8] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num8),
                    ..
                } => self.key_pad[8] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::Num9),
                    ..
                } => self.key_pad[9] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Num9),
                    ..
                } => self.key_pad[9] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => self.key_pad[10] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => self.key_pad[10] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => self.key_pad[11] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::B),
                    ..
                } => self.key_pad[11] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => self.key_pad[12] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::C),
                    ..
                } => self.key_pad[12] = true,


                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => self.key_pad[13] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => self.key_pad[13] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => self.key_pad[14] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::E),
                    ..
                } => self.key_pad[14] = false,


                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => self.key_pad[15] = true,
                Event::KeyUp {
                    keycode: Some(Keycode::F),
                    ..
                } => self.key_pad[15] = false,
                _ => {}
            }
        }

        InputAction::None
    }
}
