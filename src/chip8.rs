#![allow(dead_code)]
#![allow(non_snake_case)]

extern crate sdl2;

use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const NNN: u16 = 0x0fff;
const NN: u16 = 0x00ff;
const N: u16 = 0x000f;
const X: u16 = 0x0f00;
const Y: u16 = 0x00f0;

const WIDTH: u32 = 64; // Pixels
const HEIGHT: u32 = 32; // Pixels

const WIDTH_PER_PIXEL: u32 = 20;
const HEIGHT_PER_PIXEL: u32 = 20;

const ADDR_OFFSET: usize = 0x200;

const PIXEL_OFF_COLOR: Color = Color::RGB(0x99, 0x66, 0x01);
const PIXEL_ON_COLOR: Color = Color::RGB(0xff, 0xcc, 0x01);

struct Chip8 {
    pub stack: Vec<u16>,
    pub delay_timer: u16,
    pub sound_timer: u16,
    registers: [u8; 16],
    ip: usize, // Instruction pointer
    ireg: u16,
    rom_bytes: Vec<u8>,
    key_pad: [bool; 16],
}

impl Chip8 {
    fn standard(rom_name: &str) -> Self {
        Chip8 {
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            ip: ADDR_OFFSET,
            ireg: 0,
            rom_bytes: read_rom(rom_name),
            key_pad: [false; 16],
        }
    }

    fn push_stack(&mut self, val: u16) {
        self.stack.push(val);
    }

    fn pop_stack(&mut self) -> u16 {
        self.stack.pop().unwrap()
    }

    fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    fn draw_instr(&mut self, canv: &mut Canvas<Window>, xu16: u16, yu16: u16, height: u16) {
        let x: i32 = i32::try_from(xu16).unwrap();
        let y: i32 = i32::try_from(yu16).unwrap();

        for isprite in 0..height {
            canv.set_draw_color(PIXEL_ON_COLOR);
            let hpp_i32 = i32::try_from(HEIGHT_PER_PIXEL).unwrap();
            let wpp_i32 = i32::try_from(WIDTH_PER_PIXEL).unwrap();

            let sprite = self.rom_bytes[usize::try_from(self.ireg + isprite).unwrap()];

            let isprite_i32 = i32::try_from(isprite).unwrap();

            for bit in 0..8 {
                if (1 << (7 - bit)) & sprite != 0 {
                    canv.fill_rect(Rect::new(
                        x * wpp_i32 + bit * wpp_i32,
                        y * hpp_i32 + isprite_i32 * hpp_i32,
                        WIDTH_PER_PIXEL,
                        HEIGHT_PER_PIXEL,
                    ))
                    .unwrap();
                }
            }
        }
    }

    fn draw(&mut self, canv: &mut Canvas<Window>) {
        canv.present();
    }

    fn clear(&mut self, canv: &mut Canvas<Window>) {
        canv.set_draw_color(PIXEL_OFF_COLOR);
        canv.clear();
    }

    fn get_delay_timer(&mut self) -> u16 {
        self.delay_timer
    }

    fn fetch(&mut self) -> u16 {
        let bytes = self.get_instruction();
        self.ip += 2;

        bytes
    }

    fn skip_instructions(&mut self, num_instr: u16) {
        self.ip += usize_from_u16(num_instr * 2);
    }

    fn decode(&mut self, instr: u16, canv: &mut Canvas<Window>) {
        print_u16_hex(instr);

        let ixreg = get_X(instr);
        let iyreg = get_Y(instr);

        let xreg = self.get_X_register_value(instr);
        let yreg = self.get_Y_register_value(instr);

        match (instr & 0xf000) >> 12 {
            0 => {
                // clear screen
                match instr & NNN {
                    0xe0 => {
                        // Clear screen
                        self.clear(canv);
                    }
                    0xee => {
                        // Return to address from address in stack
                        let ret_addr = self.stack.pop().unwrap();
                        self.ip = usize::try_from(ret_addr).unwrap();
                    }
                    _ => print_unknown_instr(instr),
                }
            }
            1 => {
                // jump
                let address: u16 = instr & NNN;
                self.ip = usize_from_u16(address);
            }
            2 => {
                // jump to address and store current address in stack
                let address = instr & NNN;
                self.push_stack(u16_from_usize(self.ip));
                self.ip = usize_from_u16(address);
            }
            3 => {
                if self.register_equal(ixreg, instr & NN) {
                    self.skip_instructions(1);
                }
            }
            4 => {
                if xreg != instr & NN {
                    self.skip_instructions(1);
                }
            }
            5 => {
                // jump if registers are equal
                if xreg == yreg {
                    self.skip_instructions(1);
                }
            }
            6 => {
                // set register vx
                let value: u16 = instr & NN;

                self.set_X_register_value(instr, value);
            }
            7 => {
                // add value to register vx
                let value = instr & NN;
                self.set_X_register_value(instr, (xreg + value) & NN);
            }
            8 => {
                // binary ops
                match instr & N {
                    0x0000 => {
                        let yreg = self.get_Y_register_value(instr);

                        self.set_X_register_value(instr, yreg);
                    }
                    0x0001 => {
                        // bitwise or
                        let reg_y = self.registers[iyreg];

                        self.registers[ixreg] |= reg_y;
                    }
                    0x0002 => {
                        // bitwise and
                        let reg_y = self.registers[iyreg];
                        self.registers[ixreg] &= reg_y;
                    }
                    0x0003 => {
                        // Logical or
                        let reg_y = self.registers[iyreg];
                        self.registers[ixreg] ^= reg_y;
                    }
                    0x0004 => {
                        // Add. Also checks overflow. Sets 1 to VF if overflow
                        let reg_y = u8_from_u16(self.get_Y_register_value(instr));
                        let carry_flag_value: u8;

                        match self.registers[ixreg].checked_add(reg_y) {
                            Some(_) => {
                                carry_flag_value = 0;
                                // self.registers[0xf] = 0;
                            }
                            None => {
                                carry_flag_value = 1;
                                //self.registers[0xf] = 1;
                            }
                        }

                        self.set_X_register_value(instr, (xreg + yreg) & 0xff);
                        self.registers[0xf] = carry_flag_value;
                    } // Subtract
                    0x0005 => {
                        let reg_y = self.registers[iyreg];

                        match self.registers[ixreg].checked_sub(reg_y) {
                            Some(v) => {
                                self.registers[ixreg] = v;
                                self.registers[0xf] = 1;
                            }
                            None => {
                                let underflow_val = (0xff - (reg_y - self.registers[ixreg])) + 1;
                                self.registers[ixreg] = underflow_val;

                                self.registers[0xf] = 0;
                            }
                        }
                    }
                    0x0006 => {
                        // Ambiguous shift
                        self.set_X_register_value(instr, yreg >> 1);

                        if 0b00000001 & yreg == 0b00000001 {
                            self.registers[0xf] = 1;
                        } else if 0b00000001 & yreg == 0b00000000 {
                            self.registers[0xf] = 0;
                        }
                    }
                    0x0007 => {
                        // Subtract
                        let reg_y = self.registers[iyreg];

                        let reg_x = self.registers[ixreg];
                        match reg_y.checked_sub(reg_x) {
                            Some(v) => {
                                self.registers[ixreg] = v;
                                self.registers[0xf] = 1;
                            }
                            None => {
                                // Underflow
                                self.registers[ixreg] = (0xff - (reg_x - reg_y)) + 1;
                                self.registers[0xf] = 0;
                            }
                        }
                    }
                    0x000e => {
                        // Ambiguous shift
                        self.set_X_register_value(instr, yreg << 1);

                        if 0b10000000 & yreg == 0b10000000 {
                            self.set_register_value(0xf, 1);
                        } else if 0b10000000 & yreg == 0b00000000 {
                            self.set_register_value(0xf, 0);
                        }
                    }
                    _ => print_unknown_instr(instr),
                }
            }
            9 => {
                // jump if registers are unequal
                if xreg != yreg {
                    self.skip_instructions(1);
                }
            }
            0xa => {
                // set index register i
                let index = instr & NNN;
                self.ireg = index;
            }
            0xb => {
                // jump with offset from register v0
                let jump_addr = instr & NNN + u16_from_u8(self.registers[0]);
                self.ip = usize_from_u16(jump_addr);
            }
            0xc => {
                let val = instr & NN;

                let mut rng = rand::thread_rng();
                let rand_num: u16 = (rng.gen::<u16>() & NN) & val;

                self.set_X_register_value(instr, rand_num);
            }
            0xd => {
                // display_draw
                let height = instr & N;

                self.draw_instr(canv, xreg, yreg, height);
                self.draw(canv);
            }
            0xe => {
                // Skip if key
                match instr & NN {
                    0x009e => {
                        // Skip if key is down
                        if self.key_pad[ixreg] {
                            self.ip += 2;
                        }
                    }
                    0x00a1 => {
                        // Skip if key is not down
                        if !self.key_pad[ixreg] {
                            self.ip += 2;
                        }
                    }
                    _ => print_unknown_instr(instr),
                }
            }
            0xf => match instr & NN {
                0x0007 => {
                    let delay_timer: u16 = self.get_delay_timer();
                    self.set_X_register_value(instr, delay_timer)
                }
                0x0015 => {
                    self.delay_timer = self.get_X_register_value(instr);
                }
                0x0018 => {
                    self.sound_timer = self.get_X_register_value(instr);
                }
                0x001e => {
                    let xreg = self.get_X_register_value(instr);
                    self.ireg += xreg;
                }
                0x0033 => {
                    let ones = xreg % 10;
                    let tens = ((xreg - ones) % 100) / 10;
                    let houndreds = ((xreg - tens - ones) % 1000) / 100;

                    self.rom_bytes[usize_from_u16(self.ireg)] = u8::try_from(houndreds).unwrap();
                    self.rom_bytes[usize_from_u16(self.ireg) + 1] = u8::try_from(tens).unwrap();
                    self.rom_bytes[usize_from_u16(self.ireg) + 2] = u8::try_from(ones).unwrap();
                }
                0x0055 => {
                    for i in 0..(ixreg + 1) {
                        self.rom_bytes[usize::try_from(self.ireg).unwrap() + i] = self.registers[i];
                    }
                }
                0x0065 => {
                    for i in 0..(ixreg + 1) {
                        self.registers[i] = self.rom_bytes[usize::try_from(self.ireg).unwrap() + i];
                    }
                }
                _ => print_unknown_instr(instr),
            },
            _ => print_unknown_instr(instr),
        }
    }

    fn get_instruction(&mut self) -> u16 {
        let mut value: u16 = 0;
        let ip = self.ip;
        value += u16::try_from(self.rom_bytes[ip]).unwrap() << 8;
        value += u16::try_from(self.rom_bytes[ip + 1]).unwrap();
        return value;
    }

    fn get_register_value(&mut self, ireg: usize) -> u16 {
        u16::try_from(self.registers[ireg]).unwrap()
    }

    fn set_register_value(&mut self, ireg: usize, value: u16) {
        self.registers[ireg] = u8_from_u16(value & NN);
    }

    fn get_X_register_value(&mut self, instr: u16) -> u16 {
        u16::try_from(self.registers[usize::try_from((instr & X) >> 8).unwrap()]).unwrap()
    }

    fn get_Y_register_value(&mut self, instr: u16) -> u16 {
        u16::try_from(self.registers[usize::try_from((instr & Y) >> 4).unwrap()]).unwrap()
    }

    fn set_X_register_value(&mut self, instr: u16, val: u16) {
        self.registers[usize::try_from((instr & X) >> 8).unwrap()] =
            u8::try_from(val & NN).unwrap();
    }

    fn set_Y_register_value(&mut self, instr: u16, val: u16) {
        self.registers[usize::try_from((instr & Y) >> 4).unwrap()] =
            u8::try_from(val & NN).unwrap();
    }

    fn register_equal(&mut self, ireg: usize, val: u16) -> bool {
        self.registers[ireg] == u8::try_from(val).unwrap()
    }
}

fn get_X(instr: u16) -> usize {
    usize::try_from((instr & X) >> 8).unwrap()
}

fn get_Y(instr: u16) -> usize {
    usize::try_from((instr & Y) >> 4).unwrap()
}

fn u16_from_usize(val: usize) -> u16 {
    u16::try_from(val).unwrap()
}

fn u16_from_u8(val: u8) -> u16 {
    u16::try_from(val).unwrap()
}

fn u8_from_u16(value: u16) -> u8 {
    u8::try_from(value).unwrap()
}

fn usize_from_u16(val: u16) -> usize {
    usize::try_from(val).unwrap()
}

fn print_u16_hex(val: u16) {
    println!("{:#06x}", val);
}

fn print_unknown_instr(instr: u16) {
    println!("Unknown instruction {:#06x}", instr);
}

fn read_rom(name: &str) -> Vec<u8> {
    let raw_rom = fs::read(name).expect("Error reading rom file!");

    let mut form_rom = Vec::new();

    // Add 0x200 of empty space
    for _ in 0..ADDR_OFFSET {
        form_rom.push(0);
    }

    for i in raw_rom {
        form_rom.push(i);
    }

    form_rom
}

pub fn get_current_millis() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("ERR!");

    since_the_epoch.as_millis()
}

pub fn main_cpu_loop(rom_name: &str, instr_per_secs: f32) {
    let mut cpu: Chip8 = Chip8::standard(rom_name);

    //Canvas
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8", WIDTH * WIDTH_PER_PIXEL, HEIGHT * HEIGHT_PER_PIXEL)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .unwrap();
    cpu.clear(&mut canvas);
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut last_time: u128 = get_current_millis();
    let hz60: u128 = 1000 / 60;

    'mainloop: loop {
        // Change all keycode values to false

        for i in 0..0xf {
            cpu.key_pad[i] = false;
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => cpu.key_pad[0] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => cpu.key_pad[1] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::E),
                    ..
                } => cpu.key_pad[2] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => cpu.key_pad[3] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => cpu.key_pad[4] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => cpu.key_pad[5] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => cpu.key_pad[6] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => cpu.key_pad[7] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Z),
                    ..
                } => cpu.key_pad[8] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::X),
                    ..
                } => cpu.key_pad[9] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::C),
                    ..
                } => cpu.key_pad[10] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::V),
                    ..
                } => cpu.key_pad[11] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Num1),
                    ..
                } => cpu.key_pad[12] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Num2),
                    ..
                } => cpu.key_pad[13] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Num3),
                    ..
                } => cpu.key_pad[14] = true,
                Event::KeyDown {
                    keycode: Some(Keycode::Num4),
                    ..
                } => cpu.key_pad[15] = true,
                _ => {}
            }
        }

        let current_millis = get_current_millis();
        if current_millis - last_time >= hz60 {
            cpu.decrement_timers();
            last_time = current_millis;
        }

        let instr = cpu.fetch();
        cpu.decode(instr, &mut canvas);

        std::thread::sleep(Duration::from_secs_f32(1.0 / instr_per_secs));
    }
}
