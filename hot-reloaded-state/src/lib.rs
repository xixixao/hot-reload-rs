use hot_reload::serde::*;
use hot_reload::*;

// These are the arguments the reloadable process will need to get the shared
// state.
#[derive(Serialize, Deserialize)]
pub struct Arguments {
    pub window_width: usize,
    pub window_height: usize,
}

hot_reload!(
    // The name of the package that will be reloadable
    "example-impl",
    // The arguments passed to the reloadable package on initialization
    Arguments,
    // Template for the shared state struct
    struct HotReloaded {
        buffer: slice::<u32>(|arguments: &Arguments| {
            arguments.window_width * arguments.window_height
        }),
        signals: channel::<u32>(),
    }
);
