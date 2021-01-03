use hot_reload::simple_shared_memory::*;
use hot_reload::*;

// TODO: Consider using a macro to generate the code below, needs to handle initialization
// from owner and dependencies - or give up on dependencies
pub struct HotReloaded {
    window_len: SharedMemorySimple<usize>,
    buffer: SharedMemorySlice<u32>,
}

const APP_NAME: &str = "com.hot_reload_example";

impl HotReloaded {
    pub fn owner(
        window_width: usize,
        window_height: usize,
    ) -> Result<HotReloaded, Box<dyn std::error::Error>> {
        let (hot_reload, state) = setup(Process::Owner, window_width * window_height)?;
        *state.window_len.get() = window_width * window_height;
        hot_reload.start(
            "example-impl",
            &[&window_width.to_string(), &window_height.to_string()],
        );
        Ok(state)
    }

    pub fn reloadable() -> Result<HotReloaded, Box<dyn std::error::Error>> {
        let [window_width, window_height] = reloadable_process_args().as_slice();
        let (hot_reload, state) = setup(
            Process::Reloadable,
            window_width.parse::<usize>().unwrap() * window_height.parse::<usize>().unwrap(),
        )?;
        Ok(state)
    }
}

fn setup(
    process: Process,
    window_len: usize,
) -> Result<(HotReload, HotReloaded), Box<dyn std::error::Error>> {
    let hot_reload = HotReload::new(process);
    let state = Ok(HotReloaded {
        window_len: hot_reload.value::<usize>("window_len")?,
        buffer: hot_reload.slice::<u32>("buffer", window_len)?,
    });
    (hot_reload, state)
}
