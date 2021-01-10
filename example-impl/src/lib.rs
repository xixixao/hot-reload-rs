use arraystring::CacheString;

pub fn render() {
  let mut hot_reloaded = hot_reloaded_state::reloadable().unwrap();
  let window_len = hot_reloaded.buffer.length;
  hot_reloaded
    .buffer
    .get()
    .copy_from_slice(&vec![color((1.0, 0.7, 0.0)); window_len]);
  let mut clicks_since_start = 0;
  loop {
    let _ = hot_reloaded.channel_to_impl.recv();
    hot_reloaded.buffer.get().copy_from_slice(&vec![
      color((rand::random(), rand::random(), 0.0,));
      window_len
    ]);
    clicks_since_start += 1;
    hot_reloaded
      .channel_from_impl
      .send(&CacheString::from_str_truncate(format!(
        "{}",
        clicks_since_start
      )));
  }
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
