pub mod simple_shared_memory;

pub enum Process {
    Owner,
    Reloadable,
}

pub fn reloadable_process_args() -> Vec<String> {
    std::env::args().skip(2).collect()
}

pub struct HotReload {
    process: Process,
}

impl HotReload {
    pub fn new(process: Process) -> Self {
        HotReload { process }
    }

    pub fn value()
}

// pub fn start()

// fn new_slice<T>(
//     file_name: &str,
//     length: usize,
// ) -> Result<SharedMemorySlice<T>, Box<dyn std::error::Error>> {
//     shared_memory_with_slice::<u32>(true, "hot_reload_buffer", length).unwrap()
// }
