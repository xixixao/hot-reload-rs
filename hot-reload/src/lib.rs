pub mod simple_shared_memory;
use simple_shared_memory::*;

pub enum Process {
    Owner,
    Reloadable,
}

#[macro_export]
macro_rules! reloadable_process_args {
    ($n:literal) => {{
        use std::convert::TryInto;
        ((std::env::args().skip(2).collect::<Vec<String>>().try_into())
            as std::result::Result<[String; 2], _>)
            .map_err(|_| "Hot reload initialization failed")
    }};
}

pub struct HotReload {
    process: Process,
    shared_memory_id_prefix: String,
    reloadable_watch_process: Option<std::process::Child>,
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub type Shared<T> = Box<dyn SharedMemory<T>>;

impl HotReload {
    pub fn new(process: Process) -> Self {
        let shared_memory_id_prefix = match process {
            Process::Owner => format!("/{:X}", rand::random::<u64>()),
            Process::Reloadable => std::env::args().nth(1).unwrap(),
        };
        HotReload {
            process,
            shared_memory_id_prefix,
            reloadable_watch_process: None,
        }
    }

    pub fn start(&mut self, reloadable_process_project_name: &str, args: &[&str]) -> Result<()> {
        self.reloadable_watch_process = Some(
            std::process::Command::new("cargo")
                .args(&["watch", "-x"])
                .arg(format!(
                    "run {:?}",
                    std::process::Command::new("-p")
                        .arg(reloadable_process_project_name)
                        .arg("--")
                        .arg(&self.shared_memory_id_prefix)
                        .args(args)
                ))
                .spawn()?,
        );
        Ok(())
    }

    pub fn value<T>(&self, name: &str) -> Result<Box<impl SharedMemory<T>>> {
        Ok(Box::new(shared_memory(
            self.is_owner(),
            &self.memory_id(name),
        )?))
    }

    pub fn slice<T>(&self, name: &str, length: usize) -> Result<Box<impl SharedMemory<[T]>>> {
        Ok(Box::new(shared_memory_with_slice(
            self.is_owner(),
            &self.memory_id(name),
            length,
        )?))
    }

    fn is_owner(&self) -> bool {
        matches!(self.process, Process::Owner)
    }

    fn memory_id(&self, name: &str) -> String {
        self.shared_memory_id_prefix.clone() + name
    }
}

impl Drop for HotReload {
    fn drop(&mut self) {
        if let Some(handle) = self.reloadable_watch_process.as_mut() {
            let _ = handle.kill();
        }
    }
}
