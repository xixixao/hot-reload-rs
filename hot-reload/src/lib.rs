pub mod simple_shared_memory;
pub extern crate serde;
pub use simple_shared_memory::SharedChannel;
pub use simple_shared_memory::SharedMemory;
use simple_shared_memory::*;

pub enum Process {
    Owner,
    Reloadable,
}

pub fn reloadable_process_args<T>() -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    Ok(ron::from_str(&std::env::args().nth(2).ok_or(
        "Unexpected missing argument to reloadable process",
    )?)?)
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
            Process::Owner => format!("/{:X}", rand::random::<u32>()),
            Process::Reloadable => std::env::args().nth(1).unwrap(),
        };
        HotReload {
            process,
            shared_memory_id_prefix,
            reloadable_watch_process: None,
        }
    }

    pub fn start<TArgs>(
        &mut self,
        reloadable_process_project_name: &str,
        serialized_args: &TArgs,
    ) -> Result<()>
    where
        TArgs: serde::Serialize,
    {
        self.reloadable_watch_process = Some(
            std::process::Command::new("cargo")
                .args(&["watch", "-x"])
                .arg(format!(
                    "run {:?}",
                    std::process::Command::new("-p")
                        .arg(reloadable_process_project_name)
                        .arg("--")
                        .arg(&self.shared_memory_id_prefix)
                        .arg(&ron::to_string(serialized_args)?)
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

    pub fn channel<T>(&self, name: &str) -> Result<Box<SharedChannel<T>>>
    where
        T: Copy,
    {
        Ok(Box::new(shared_channel(
            self.is_owner(),
            &self.memory_id(name),
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

/// Creates a new shared state between `owner` and `reloadable` processes.
///
/// Any data in the state must not contain pointers. It is the same as the data that
/// would be allocated on the stack (unfortunately there is no Trait for this
/// constraint in Rust).
///
/// # Examples
///
/// ```
/// hot_reload!(
///    // The name of the package that will be reloadable
///    "example-impl",
///    // The arguments passed to the reloadable package on initialization
///    Arguments,
///    // Template for the shared state struct
///    struct HotReloaded {
///        buffer: slice::<u32>(|arguments: &Arguments| {
///            arguments.window_width * arguments.window_height
///        }),
///        channel_to_impl: channel::<()>(),
///        channel_from_impl: channel::<str>(),
///    }
/// );
/// ```
#[macro_export]
macro_rules! hot_reload {
    (
        $project_name:literal,
        $args_type_name:ident,
        struct $state_struct_name:ident
            {
                $($field_name:ident : $field_type:ident::<$field_type_arg:ty>
                    ( $( $declarator:expr)? )
                ),+ $(,)?
            }) => {
        pub struct $state_struct_name {
            hot_reload: HotReload,
            $(pub $field_name: $crate::hot_reload_field_type!($field_type $field_type_arg),)+
        }

        pub fn owner(arguments: $args_type_name) -> Result<$state_struct_name> {
            let mut state = setup(Process::Owner, &arguments)?;
            state.hot_reload.start($project_name, &arguments)?;
            Ok(state)
        }

        pub fn reloadable() -> Result<$state_struct_name> {
            setup(Process::Reloadable, &reloadable_process_args()?)
        }

        fn setup(
            process: Process,
            arguments: &$args_type_name,
        ) -> Result<$state_struct_name> {
            let hot_reload = HotReload::new(process);

            $(
                let $field_name = $crate::hot_reload_field_definition!(
                    $field_type,
                    hot_reload,
                    arguments,
                    $field_name,
                    $($declarator)?
                );
            )+

            // let buffer = hot_reload.slice::<u32>("buffer", window_width * window_height)?;
            Ok($state_struct_name {
                hot_reload,
                $($field_name,)+
                // buffer
            })
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! hot_reload_field_definition {
    (
        slice,
        $reload_instance_name:ident,
        $args_variable_name:ident,
        $field_name:ident,
        $declarator:expr) => {{
        $reload_instance_name.slice(stringify!($field_name), $declarator($args_variable_name))?
    }};
    (
        value,
        $reload_instance_name:ident,
        $args_variable_name:ident,
        $field_name:ident,) => {{
        $reload_instance_name.value(stringify!($field_name))?
    }};
    (
        channel,
        $reload_instance_name:ident,
        $args_variable_name:ident,
        $field_name:ident, ) => {{
        $reload_instance_name.channel(stringify!($field_name))?
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! hot_reload_field_type {
    (slice $type_arg:ty) => {
        Box<dyn SharedMemory<[$type_arg]>>
    };
    (value $type_arg:ty) => {
        Box<dyn SharedMemory<$type_arg>>
    };
    (channel $type_arg:ty) => {
        Box<SharedChannel<$type_arg>>
    };
}
