use hot_reload::simple_shared_memory::*;
use hot_reload::*;

pub struct HotReloaded {
    hot_reload: HotReload,
    pub buffer: Box<dyn SharedMemory<[u32]>>,
}

pub fn owner(window_width: usize, window_height: usize) -> Result<HotReloaded> {
    let mut state = setup(Process::Owner, window_width * window_height)?;
    state.hot_reload.start(
        "example-impl",
        &[&window_width.to_string(), &window_height.to_string()],
    )?;
    Ok(state)
}

pub fn reloadable() -> Result<HotReloaded> {
    let [window_width, window_height] = reloadable_process_args!(2)?;
    setup(
        Process::Reloadable,
        window_width.parse::<usize>().unwrap() * window_height.parse::<usize>().unwrap(),
    )
}

fn setup(process: Process, window_len: usize) -> Result<HotReloaded> {
    let hot_reload = HotReload::new(process);
    let buffer = hot_reload.slice::<u32>("buffer", window_len)?;
    Ok(HotReloaded { hot_reload, buffer })
}
