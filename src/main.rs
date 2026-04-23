// based on: https://austinmorlan.com/posts/chip8_emulator/
// and https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#specifications

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use minifb::Key;
use chip_8::Chip8;

fn main() {
    let mut c = Chip8::default();
    c.load_rom("roms/test_opcode.ch8");

    let c = Arc::new(Mutex::new(c));
    let mut window = chip_8::display::init_window();

    {
        const TIMER_HZ: u8 = 60;
        let timer_state = Arc::clone(&c);
        thread::spawn(move || loop {
            {
                let mut chip8 = timer_state.lock().expect("Mutex poisoned in timer thread");
                chip8.decrease_timers();
            }
            thread::sleep(Duration::from_micros(1_000_000 / TIMER_HZ as u64));
        });
    }

    {
        const INSTRUCTIONS_PER_SECOND: u16 = 700;
        let timer_state = Arc::clone(&c);
        thread::spawn(move || loop {
            {
                let mut chip8 = timer_state.lock().expect("Mutex poisoned in timer thread");
                chip8.tick();
            }
            thread::sleep(Duration::from_micros(1_000_000 / INSTRUCTIONS_PER_SECOND as u64));
        });
    }



    while window.is_open() && !window.is_key_down(Key::Escape) {
        let display = {
            let c = c.lock().expect("Threads should not lock up");
            c.display
        };
        window
            .update_with_buffer(&display, chip_8::display::WIDTH, chip_8::display::HEIGHT)
            .unwrap();
    }
}