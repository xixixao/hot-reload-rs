use minifb::{Key, Window, WindowOptions};

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

    // let mut buffer = hot_reloaded.buffer();
    //     shared_memory_with_slice::<u32>(true, "hot_reload_buffer", window_len).unwrap();
    // Initialize if we want to
    // buffer
    //     .get()
    //     .copy_from_slice(&vec![color((1.0, 0.7, 0.0)); window_len]);

    // #[cfg(not(feature = "hot_reload"))]
    // render(buffer.get());

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(hot_reloaded.buffer.get(), window_width, window_height)
            .unwrap();
    }
    Ok(())
}
