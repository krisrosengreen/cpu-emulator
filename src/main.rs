mod chip8;

const INSTR_PER_SECS: f32 = 600.0;

fn main() {
    // chip8::main_cpu_loop("chip8-roms/programs/17.ch8", INSTR_PER_SECS);
    chip8::main_cpu_loop("roms/5-quirks.ch8", INSTR_PER_SECS);
}
