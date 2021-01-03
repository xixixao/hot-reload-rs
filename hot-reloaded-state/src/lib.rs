use hot_reload::serde::*;
use hot_reload::*;

pub struct HotReloaded {
    // This will take care of killing the "child" watch process when we close
    // the owner process.
    hot_reload: HotReload,
    // The list of shared state follows:
    pub buffer: Box<dyn SharedMemory<[u32]>>,
}

// These are the arguments the reloadable process will need to get the shared
// state.
#[derive(Serialize, Deserialize)]
pub struct Arguments {
    pub window_width: usize,
    pub window_height: usize,
}

// This initiates the shared state and starts the child hot reload watch process
pub fn owner(arguments: Arguments) -> Result<HotReloaded> {
    let mut state = setup(Process::Owner, &arguments)?;
    state.hot_reload.start("example-impl", &arguments)?;
    Ok(state)
}

// This gets the shared state from the reloadable process
pub fn reloadable() -> Result<HotReloaded> {
    setup(Process::Reloadable, &reloadable_process_args()?)
}

// Implementation of the shared state using HotReload.
fn setup(
    process: Process,
    Arguments {
        window_width,
        window_height,
    }: &Arguments,
) -> Result<HotReloaded> {
    let hot_reload = HotReload::new(process);
    let buffer = hot_reload.slice::<u32>("buffer", window_width * window_height)?;
    Ok(HotReloaded { hot_reload, buffer })
}
