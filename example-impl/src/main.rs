use hot_reload::simple_shared_memory::*;

fn main() {
    let window_width = 300;
    let window_height = 300;
    let window_len = window_width * window_height;
    let mut buffer =
        shared_memory_with_slice::<u32>(false, "hot_reload_buffer", window_len).unwrap();
    buffer
        .get()
        .copy_from_slice(&vec![color((1.0, 0.7, 0.0)); window_len]);
}

pub fn color((r, g, b): (f64, f64, f64)) -> u32 {
    color_256_u32(r * 255.999, g * 255.999, b * 255.999, 255.0)
}

pub fn color_256_u32(red: f64, green: f64, blue: f64, alpha: f64) -> u32 {
    ((alpha as u32) << 24) + // shift to align
     ((red as u32) << 16) + // shift to align
     ((green as u32) << 8) + // shift to align
     (blue as u32)
}
