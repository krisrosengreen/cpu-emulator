mod cpu;
mod display;
mod input;

const INSTR_PER_SECS: f32 = 100.0;

fn main() {
    cpu::main_cpu_loop("roms/2-ibm-logo.ch8", INSTR_PER_SECS);
}
