#![allow(dead_code)]

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

const INSTR_PER_SECS: f32 = 20.0;


struct Chip8 {
    pub stack: Vec<u16>,
    pub delay_timer: i32,
    pub sound_timer: i32,
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
        if self.delay_timer == 0
        {
            self.delay_timer = 60;
        }

        if self.sound_timer == 0
        {
            self.sound_timer = 60;
        }

        self.delay_timer -= 1;
        self.sound_timer -= 1;
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
                    _ => println!("Bad instruction!")
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
                let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                let reg_x = self.registers[ireg_x];

                if reg_x != instr & NN {
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
                let register: usize = usize::try_from((instr & X) >> 8).unwrap();
                let value = instr & NN;
                self.registers[register] += value;
            },
            8 => { // binary ops
                match instr & N {
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
                    4 => { // Add
                        let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                        let reg_y = self.registers[ireg_y];

                        let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                        self.registers[ireg_x] += reg_y;
                    }, // Subtract
                    5 => {
                        let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                        let reg_y = self.registers[ireg_y];

                        let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                        self.registers[ireg_x] -= reg_y;
                    },
                    7 => { // Subtract
                        let ireg_y = usize::try_from((instr & Y) >> 4).unwrap();
                        let reg_y = self.registers[ireg_y];
                    
                        let ireg_x = usize::try_from((instr & X) >> 8).unwrap();
                        let reg_x = self.registers[ireg_x];
                        self.registers[ireg_x] = reg_y - reg_x;
                    },
                    8 => {
                        // Incomplete!
                    }
                    _ => println!("Unknown opcode!")
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
            }
            _ => println!("Do not recognize the opcode")
        }
    }


    fn get_instruction(&mut self) -> u16{
        let mut value: u16 = 0;
        let ip = self.ip;
        value += u16::try_from(self.rom_bytes[ip]).unwrap() << 8;
        value += u16::try_from(self.rom_bytes[ip + 1]).unwrap();
        return value
    }
}


fn print_u16_hex(val: u16) {
    println!("{:#06x}", val);
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


pub fn main_cpu_loop(rom_name: &str) {
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

    'mainloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }
        // Decrement timers
        cpu.decrement_timers();

        let instr = cpu.fetch();
        cpu.decode(instr, &mut canvas);

        std::thread::sleep(Duration::from_secs_f32(1.0/INSTR_PER_SECS));
    }
}
