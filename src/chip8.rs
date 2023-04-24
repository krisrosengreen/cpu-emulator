#![allow(dead_code)]
#![allow(non_snake_case)]

extern crate sdl2;

use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::fs;
use std::time::{Duration, UNIX_EPOCH, SystemTime};
use rand::Rng;

const NNN: u16 = 0x0fff;
const NN: u16 = 0x00ff;
const N: u16 = 0x000f;
const X: u16 = 0x0f00;
const Y: u16 = 0x00f0;

const WIDTH: u32 = 64;  // Pixels
const HEIGHT: u32 = 32;  // Pixels

const WIDTH_PER_PIXEL: u32 = 20;
const HEIGHT_PER_PIXEL: u32 = 20;

const ADDR_OFFSET: u16 = 0x200;



struct Chip8 {
    pub stack: Vec<u16>,
    pub delay_timer: u16,
    pub sound_timer: u16,
    registers: [u16; 16],
    ip: usize,  // Instruction pointer
    ireg: u16,
    rom_bytes: Vec<u8>
}


impl Chip8 {
    // The stack
    fn push_stack(&mut self, val: u16) {
        self.stack.push(val);
    }


    fn pop_stack(&mut self) -> u16 {
        self.stack.pop().unwrap()
    }


    //Timers
    fn decrement_timers(&mut self) {
        if self.delay_timer > 0
        {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0
        {
            self.sound_timer -= 1;
        }
    }


    fn draw_instr(&mut self, canv: &mut Canvas<Window>, xu16: u16, yu16: u16, height: u16) {

        let x: i32 = i32::try_from(xu16).unwrap();
        let y: i32 = i32::try_from(yu16).unwrap();
        
        for isprite in 0..height {
            canv.set_draw_color(Color::RGB(255, 255, 255));
            let hpp_i32 = i32::try_from(HEIGHT_PER_PIXEL).unwrap();
            let wpp_i32 = i32::try_from(WIDTH_PER_PIXEL).unwrap();

            let sprite = self.rom_bytes[usize::try_from(self.ireg + isprite).unwrap()];

            let isprite_i32 = i32::try_from(isprite).unwrap();

            for bit in 0..8 {
                if (1 << (7 - bit)) & sprite != 0 {
                    canv.fill_rect(Rect::new(x*wpp_i32 + bit*wpp_i32, y*hpp_i32 + isprite_i32*hpp_i32, WIDTH_PER_PIXEL, HEIGHT_PER_PIXEL)).unwrap();
                } 
            }
        }
    }


    fn draw(&mut self, canv: &mut Canvas<Window>) {
        canv.present();
    }


    fn clear(&mut self, canv: &mut Canvas<Window>) {
        canv.set_draw_color(Color::RGB(0, 0, 0));
        canv.clear();
    }


    fn get_delay_timer(&mut self) -> u16 {
        let hz60: u128 = 1000/60;
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("ERR!");

        let millis = since_the_epoch.as_millis();
        u16::try_from((millis / hz60) % 60).unwrap()
    }


    //Fetch
    fn fetch(&mut self) -> u16 {
        // Read two bytes from memory and combine into one
        // 16-bit instruction.

        // Incement by 2 bytes and be ready to fetch next opcode
        let bytes = self.get_instruction();
        self.ip += 2;

        bytes
    }

    
    fn skip_instructions(&mut self, num_instr: u16) {
        self.ip += usize::try_from(num_instr*2).unwrap();
    }


    //Decode
    fn decode(&mut self, instr: u16, canv: &mut Canvas<Window>) {
        print_u16_hex(instr);

        match (instr & 0xf000) >> 12 {
            0 => {  // clear screen
                match instr & NNN {
                    0xe0 => { // Clear screen
                        self.clear(canv);
                    },
                    0xee => { // Return to address from address in stack
                        let ret_addr = self.stack.pop().unwrap();
                        self.ip = usize::try_from(ret_addr).unwrap();
                    }
                    _ => print_unknown_instr(instr)
                }
            },
            1 => {  // jump
                let address: u16 = instr & NNN;
                self.ip = usize::try_from(address).unwrap();
            },
            2 => { // jump to address and store current address in stack
                let address = instr & NNN;
                self.push_stack(u16::try_from(self.ip).unwrap());
                self.ip = usize::try_from(address).unwrap();
            },
            3 => {
                let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                let reg_x = self.registers[ireg_x];

                if reg_x == instr & NN {
                    self.skip_instructions(1);
                }
            },
            4 => {
                let xreg = self.get_X_register_value(instr);

                if xreg != instr & NN {
                    self.skip_instructions(1);
                }
            },
            5 => { // jump if registers are equal
                let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                let reg_x = self.registers[ireg_x];

                let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                let reg_y = self.registers[ireg_y];

                if reg_x == reg_y {
                    self.skip_instructions(1);
                }
            },
            6 => {  // set register vx
                let register: usize = usize::try_from((instr & X) >> 8).unwrap();
                let value: u16 = instr & NN;
                self.registers[register] = value;
            },
            7 => {  // add value to register vx
                let xreg = self.get_X_register_value(instr);
                let value = instr & NN;
                self.set_X_register_value(instr, xreg + value);
                println!("xreg value {} value add {} new xreg value {}", xreg, value, self.get_X_register_value(instr));
            },
            8 => { // binary ops
                match instr & N {
                    0 => {
                        let xreg = get_X(instr);
                        let yreg = get_Y(instr);

                        self.set_X_register_value(instr, u16_from_usize(xreg | yreg));
                    },
                    1 => { // bitwise or
                        let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                        let reg_y = self.registers[ireg_y];

                        let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                        self.registers[ireg_x] |= reg_y;
                    },
                    2 => { // bitwise and
                        let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                        let reg_y = self.registers[ireg_y];

                        let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                        self.registers[ireg_x] &= reg_y;
                    },
                    3 => { // Logical or
                        let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                        let reg_y = self.registers[ireg_y];

                        let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                        self.registers[ireg_x] ^= reg_y;
                    },
                    4 => { // Add. Also checks overflow. Sets 1 to VF if overflow
                        let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                        let reg_y = self.registers[ireg_y];

                        let ireg_x = usize::try_from((instr & X) >> 8).unwrap();

                        match self.registers[ireg_x].checked_add(reg_y) {
                            Some(v) => {
                                self.registers[ireg_x] = v;
                                self.registers[0xf] = 0;
                            },
                            None => {
                                self.registers[0xf] = 1;
                            }
                        }
                    }, // Subtract
                    5 => {
                        let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                        let reg_y = self.registers[ireg_y];

                        let ireg_x = usize::try_from((instr & X) >> 8).unwrap();

                        match self.registers[ireg_x].checked_sub(reg_y) {
                            Some(v) => {
                                self.registers[ireg_x] = v;
                                self.registers[0xf] = 1;
                            },
                            None => {
                                self.registers[0xf] = 1; 
                            }
                        }
                    },
                    6 => { // Ambiguous shift
                        let yreg_val = self.get_Y_register_value(instr);

                        if 0xf000 & yreg_val == 0x0001 {
                            self.registers[0xf] = 1;
                        } else if 0xf000 & yreg_val == 0x0000 {
                            self.registers[0xf] = 0;
                        }

                        self.set_X_register_value(instr, yreg_val >> 1);
                    },
                    7 => { // Subtract
                        let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                        let reg_y = self.registers[ireg_y];
                    
                        let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                        let reg_x = self.registers[ireg_x];
                        match reg_y.checked_sub(reg_x) {
                            Some(v) => {
                                self.registers[ireg_x] = v;
                                self.registers[0xf] = 1;
                            },
                            None => {
                                self.registers[0xf] = 1; 
                            }
                        }
                    },
                    0xe => { // Ambiguous shift
                        let yreg_val = self.get_Y_register_value(instr);

                        if 0xf000 & yreg_val == 0x1000 {
                            self.registers[0xf] = 1;
                        } else if 0xf000 & yreg_val == 0x0000 {
                            self.registers[0xf] = 0;
                        }

                        self.set_X_register_value(instr, yreg_val << 1);
                    },
                    _ => print_unknown_instr(instr)
                }
            },
            9 => { // jump if registers are unequal
                let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                let reg_x = self.registers[ireg_x];
                
                let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                let reg_y = self.registers[ireg_y];
                    
                if reg_x != reg_y {
                    self.skip_instructions(1);
                }
            },
            0xa => {  // set index register i
                let index = instr & NNN;
                self.ireg = index;
            },
            0xb => { // jump with offset from register v0
                let jump_addr = instr & NNN + self.registers[0];
                self.ip = usize::try_from(jump_addr).unwrap();
            },
            0xc => {
                let val = instr & NN;

                let mut rng = rand::thread_rng();
                let rand_num: u16 = (rng.gen::<u16>() & NN) & val;

                let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                self.registers[ireg_x] = rand_num;
            },
            0xd => {  // display_draw
                let reg_x = usize::try_from((instr & X) >> 8).unwrap();
                let reg_y = usize::try_from((instr & Y) >> 4).unwrap();
                let height = instr & N;
                
                self.draw_instr(canv, self.registers[reg_x], self.registers[reg_y], height);
                self.draw(canv);
            },
            0xf => {
                match instr & NN {
                    0x0007 => {
                        self.registers[get_X(instr)] = self.get_delay_timer();
                    },
                    0x0015 => {
                        let ireg = get_X(instr);

                        self.delay_timer = self.registers[ireg];
                    },
                    0x0018 => {
                        let ireg = get_X(instr);

                        self.sound_timer = self.registers[ireg];
                    },
                    0x001e => {
                        let xreg = self.get_X_register_value(instr);
                        self.ireg += xreg;
                    },
                    0x0033 => {
                        let xreg = self.get_X_register_value(instr);

                        let ones = xreg % 10;
                        let tens = ((xreg - ones) % 100) / 10;
                        let houndreds = ((xreg - tens - ones) % 1000) / 100;

                        self.rom_bytes[usize_from_u16(self.ireg)] = u8::try_from(ones).unwrap();
                        self.rom_bytes[usize_from_u16(self.ireg) + 1] = u8::try_from(tens).unwrap();
                        self.rom_bytes[usize_from_u16(self.ireg) + 2] = u8::try_from(houndreds).unwrap();
                    },
                    0x0055 => {
                        let xval = get_X(instr);

                        for i in 0..(xval + 1) {
                            self.rom_bytes[usize::try_from(self.ireg).unwrap() + i*2] = u8::try_from(self.registers[i] & NN).unwrap();
                            self.rom_bytes[usize::try_from(self.ireg + 1).unwrap() + i*2] = u8::try_from(self.registers[i] >> 8).unwrap();
                        }
                    },
                    0x0065 => {
                        let xval = get_X(instr);

                        for i in 0..(xval + 1) {
                            let mut two_byte_value: u16 = u16::try_from(self.rom_bytes[usize::try_from(self.ireg).unwrap() + i*2]).unwrap();
                            two_byte_value += u16::try_from(self.rom_bytes[usize::try_from(self.ireg + 1).unwrap() + i*2]).unwrap() << 8;

                            self.registers[i] = two_byte_value;
                        }
                        
                    },
                    _ => print_unknown_instr(instr)
                }
            }
            _ => print_unknown_instr(instr)
        }
    }


    fn get_instruction(&mut self) -> u16{
        let mut value: u16 = 0;
        let ip = self.ip;
        value += u16::try_from(self.rom_bytes[ip]).unwrap() << 8;
        value += u16::try_from(self.rom_bytes[ip + 1]).unwrap();
        return value
    }


    fn get_X_register_value(&mut self, instr: u16) -> u16 {
        self.registers[usize::try_from((instr&X)>>8).unwrap()]
    }


    fn get_Y_register_value(&mut self, instr: u16) -> u16 {
        self.registers[usize::try_from((instr&Y)>>4).unwrap()]
    }


    fn set_X_register_value(&mut self, instr: u16, val: u16)  {
        self.registers[usize::try_from((instr&X)>>8).unwrap()] = val;
    }


    fn set_Y_register_value(&mut self, instr: u16, val: u16) {
        self.registers[usize::try_from((instr&Y)>>4).unwrap()] = val;
    }

}


fn get_X(instr: u16) -> usize {
    usize::try_from((instr & X) >> 8).unwrap()
}


fn get_Y(instr: u16) -> usize {
    usize::try_from((instr & X) >> 4).unwrap()
}


fn u16_from_usize(val: usize) -> u16 {
    u16::try_from(val).unwrap()
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


fn read_rom(name: &str) -> Vec<u8>{
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
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("ERR!");

    let millis = since_the_epoch.as_millis();
    
    millis
}


pub fn main_cpu_loop(rom_name: &str, instr_per_secs: f32) {
    let mut cpu: Chip8 = Chip8 {
        stack: Vec::new(),
        delay_timer: 60,
        sound_timer: 60,
        registers: [0; 16],
        ireg: 0,
        ip: usize::try_from(ADDR_OFFSET).unwrap(),
        rom_bytes: read_rom(rom_name)
    };

    //Canvas
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("Chip-8", WIDTH*WIDTH_PER_PIXEL, HEIGHT*HEIGHT_PER_PIXEL)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_draw_color(Color::RGB(255, 255, 255));

    let hz60: u128 = 1000/60;

    // Timers
    
    let mut last_time: u128 = get_current_millis();

    'mainloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }

        let current_millis = get_current_millis();

        if current_millis - last_time >= hz60 {
            cpu.decrement_timers();
            last_time = current_millis;
        }

        // Decrement timers

        let instr = cpu.fetch();
        cpu.decode(instr, &mut canvas);

        std::thread::sleep(Duration::from_secs_f32(1.0/instr_per_secs));
    }
}
