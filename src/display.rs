use minifb::{Scale, Window, WindowOptions};

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub fn init_window() -> Window {

    let mut window_opt = WindowOptions::default();
    window_opt.scale = Scale::X16;

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        window_opt,
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(60);

    window
}
