mod chip8; 

const INSTR_PER_SECS: f32 = 20.0;


fn main() {
    // chip8::main_cpu_loop("chip8-roms/programs/", INSTR_PER_SECS);
    chip8::main_cpu_loop("roms/3-corax+.ch8", INSTR_PER_SECS);
}
