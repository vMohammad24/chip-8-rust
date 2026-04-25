use minifb::{Scale, Window, WindowOptions};

pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 64;

pub fn init_window() -> Window {
    let mut window = Window::new(
        "Chip 8 Emulator - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X8,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);

    window
}
