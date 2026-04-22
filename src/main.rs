// based on: https://austinmorlan.com/posts/chip8_emulator/
// and https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#specifications

use minifb::Key;
use chip_8::Chip8;

fn main() {
    let mut c = Chip8::new();
    c.load_rom("roms/Chip8 Picture.ch8");

    let mut window = chip_8::display::init_window();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&c.display, chip_8::display::WIDTH, chip_8::display::HEIGHT)
            .unwrap();
    }
}