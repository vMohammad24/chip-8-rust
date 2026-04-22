// based on: https://austinmorlan.com/posts/chip8_emulator/
// and https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#specifications

use chip_8::Chip8;

fn main() {
    let mut c = Chip8::new();
    c.load_rom("roms/Chip8 Picture.ch8");

}
