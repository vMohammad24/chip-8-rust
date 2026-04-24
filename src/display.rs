use minifb::{Scale, Window, WindowOptions};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub fn init_window() -> Window {
    let mut window = Window::new(
        "Chip 8 Emulator - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(0);

    window
}
