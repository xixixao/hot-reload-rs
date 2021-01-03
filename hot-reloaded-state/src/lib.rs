use hot_reload::simple_shared_memory::*;
use hot_reload::*;

pub struct HotReloaded {
    // This will take care of killing the "child" watch process when we close
    // the owner process.
    hot_reload: HotReload,
    // The list of shared state follows:
    pub buffer: Box<dyn SharedMemory<[u32]>>,
}

// This initiates the shared state and starts the child hot reload watch process
pub fn owner(window_width: usize, window_height: usize) -> Result<HotReloaded> {
    let mut state = setup(Process::Owner, window_width * window_height)?;
    state.hot_reload.start(
        "example-impl",
        &[&window_width.to_string(), &window_height.to_string()],
    )?;
    Ok(state)
}

// This gets the shared state from the reloadable process
pub fn reloadable() -> Result<HotReloaded> {
    let [window_width, window_height] = reloadable_process_args!(2)?;
    setup(
        Process::Reloadable,
        window_width.parse::<usize>()? * window_height.parse::<usize>()?,
    )
}

// Implementation of the shared state using HotReload.
fn setup(process: Process, window_len: usize) -> Result<HotReloaded> {
    let hot_reload = HotReload::new(process);
    let buffer = hot_reload.slice::<u32>("buffer", window_len)?;
    Ok(HotReloaded { hot_reload, buffer })
}
