use chip_8::cpu::Chip8;
use chip_8::keys::get_keypad;
use minifb::Key;
use rfd::FileDialog;
use rodio::{DeviceSinkBuilder, Player};
use std::env;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn main() {
    let mut args = env::args();
    let mut chip = Chip8::default();

    let rom = match args.nth(1) {
        Some(file) => PathBuf::from(file),
        None => {
            let Some(path) = FileDialog::new()
                .set_title("Select Chip-8 Rom File")
                .set_directory(env::current_dir().unwrap_or_default())
                .pick_file()
            else {
                println!("Please Select a file.");
                return;
            };
            path
        }
    };

    chip.load_rom(rom);

    let mut window = chip_8::display::init_window();

    // audio
    let mut sink =
        DeviceSinkBuilder::open_default_sink().expect("Failed to open default audio stream");
    let player = Player::connect_new(&sink.mixer());
    let beep = rodio::source::SineWave::new(440.0);
    player.append(beep);
    player.pause();
    sink.log_on_drop(false);

    const CPU_HZ: f64 = 700.0;
    const TIMER_HZ: f64 = 60.0;
    const MAX_DELTA: Duration = Duration::from_millis(100);
    const CPU_TICK_RATE: Duration = Duration::from_micros((1_000_000.0 / CPU_HZ) as u64);
    const TIMER_TICK_RATE: Duration = Duration::from_micros((1_000_000.0 / TIMER_HZ) as u64);

    let mut last_time = Instant::now();
    let mut cpu_accumulator = Duration::ZERO;
    let mut timer_accumulator = Duration::ZERO;
    let mut prev_display = [0u32; chip_8::display::WIDTH * chip_8::display::HEIGHT];
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        let mut delta = now.duration_since(last_time);
        // clamp delta so if the thread is ever frozen the cpu doesnt explode
        if delta > MAX_DELTA {
            delta = MAX_DELTA;
        }

        last_time = now;

        cpu_accumulator += delta;
        timer_accumulator += delta;
        chip.keypad = get_keypad(&window);

        while timer_accumulator >= TIMER_TICK_RATE {
            chip.decrease_timers();
            timer_accumulator -= TIMER_TICK_RATE;
        }

        while cpu_accumulator >= CPU_TICK_RATE {
            chip.tick();
            cpu_accumulator -= CPU_TICK_RATE;
        }

        if chip.sound_timer > 0 && player.is_paused() {
            player.play();
        } else if chip.sound_timer == 0 && !player.is_paused() {
            player.pause();
        }

        if chip.dirty {
            chip.dirty = false;
            let mut render_buffer = [0u32; chip_8::display::WIDTH * chip_8::display::HEIGHT];

            for i in 0..render_buffer.len() {
                // combine the current frame with the previous' ghost (basic motion blur to replicate old crt)
                render_buffer[i] = chip.display[i] | prev_display[i];
            }

            prev_display = chip.display;

            window
                .update_with_buffer(
                    &render_buffer,
                    chip_8::display::WIDTH,
                    chip_8::display::HEIGHT,
                )
                .unwrap();
        } else {
            window.update();
        }
    }
}
