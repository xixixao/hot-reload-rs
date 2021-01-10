use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};

const TARGET_FPS: f64 = 60.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window_width = 300;
    let window_height = 300;
    let mut window = Window::new(
        "Raytracer",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_secs_f64(1.0 / TARGET_FPS)));

    let mut hot_reloaded = hot_reloaded_state::owner(hot_reloaded_state::Arguments {
        window_width,
        window_height,
    })?;

    let mut mouse_was_down = false;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if window.get_mouse_pos(MouseMode::Discard).is_some() {
            let is_mouse_down = window.get_mouse_down(MouseButton::Left);
            if !is_mouse_down && mouse_was_down {
                hot_reloaded.signals.send(&1);
            }
            mouse_was_down = is_mouse_down;
        }
        window
            .update_with_buffer(hot_reloaded.buffer.get(), window_width, window_height)
            .unwrap();
    }
    Ok(())
}
