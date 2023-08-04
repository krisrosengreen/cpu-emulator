mod chip;
mod display;
mod input;

const INSTR_PER_SECS: f32 = 100.0;

fn main() {
    chip::main_chip_loop("roms/2-ibm-logo.ch8", INSTR_PER_SECS);
}
