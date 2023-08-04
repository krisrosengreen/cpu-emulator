mod chip;
mod display;
mod input;

const INSTR_PER_SECS: f32 = 500.0;

fn main() {
    chip::main_chip_loop("roms/5-quirks.ch8", INSTR_PER_SECS);
}
