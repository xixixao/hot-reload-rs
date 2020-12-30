// use raw_sync::locks::*;
use raw_sync::locks::*;
use shared_memory::*;

// Provides a cross-process channel with a familiar API, similar to [`std::sync::mpsc::channel`].
pub fn shared_channel<T>(
    is_owner: bool,
    file_name: &str,
) -> Result<SharedChannel<T>, Box<dyn std::error::Error>>
where
    T: Copy,
{
    Ok(SharedChannel {
        memory: shared_memory_with_mutex::<SharedChannelInternal<T>>(is_owner, file_name)?,
    })
}

pub struct SharedChannel<T>
where
    T: Copy,
{
    memory: SharedMemoryWithMutex<SharedChannelInternal<T>>,
}

struct SharedChannelInternal<T> {
    value: Option<T>,
}

impl<T> SharedChannel<T>
where
    T: Copy,
{
    pub fn try_recv(&mut self) -> Option<T> {
        let mut internal = self.memory.get();
        if let Some(value) = internal.value {
            internal.value = None;
            return Some(value);
        }
        None
    }

    pub fn send(&mut self, data: &T)
    where
        T: Copy,
    {
        let internal = self.memory.get();
        internal.value = Some(*data);
    }
}

pub trait SharedMemory<T: ?Sized> {
    fn get(&mut self) -> &mut T;
}

// This is a type-safe (but not synchronized) API for writing to shared memory.
pub struct SharedMemorySimple<T> {
    memory: Shmem,
    memory_type: std::marker::PhantomData<T>,
}

impl<T> SharedMemory<T> for SharedMemorySimple<T> {
    fn get(&mut self) -> &mut T {
        unsafe { &mut *(self.memory.as_ptr() as *mut T) }
    }
}

// This is a type-safe (but not synchronized) API for writing to shared memory.
pub struct SharedMemorySlice<T> {
    pub memory: Shmem,
    memory_type: std::marker::PhantomData<T>,
    length: usize,
}

impl<T> SharedMemory<[T]> for SharedMemorySlice<T> {
    fn get(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(&mut *(self.memory.as_ptr() as *mut T), self.length)
        }
    }
}

// This is a type-safe and synchronized API for writing to shared memory.
pub struct SharedMemoryWithMutex<T> {
    #[allow(dead_code)]
    memory: Shmem,
    memory_type: std::marker::PhantomData<T>,
    mutex: Box<dyn LockImpl>,
}

impl<T> SharedMemory<T> for SharedMemoryWithMutex<T> {
    fn get(&mut self) -> &mut T {
        let guard = self.mutex.lock().unwrap();
        unsafe { &mut *(*guard as *mut T) }
    }
}

// Provides a shared memory between two processes, without synchronization, with a size known
// at compile time.
pub fn shared_memory<T>(
    is_owner: bool,
    file_name: &str,
) -> Result<SharedMemorySimple<T>, Box<dyn std::error::Error>> {
    Ok(SharedMemorySimple {
        memory: get_shared_memory(is_owner, file_name, std::mem::size_of::<T>())?,
        memory_type: std::marker::PhantomData,
    })
}

// Provides a shared memory between two processes, without synchronization, with dynamic size.
pub fn shared_memory_with_slice<T>(
    is_owner: bool,
    file_name: &str,
    length: usize,
) -> Result<SharedMemorySlice<T>, Box<dyn std::error::Error>> {
    Ok(SharedMemorySlice {
        memory: get_shared_memory(is_owner, file_name, std::mem::size_of::<T>() * length)?,
        memory_type: std::marker::PhantomData,
        length,
    })
}

// Provides a shared memory between two processes, with synchronization.
pub fn shared_memory_with_mutex<T>(
    is_owner: bool,
    file_name: &str,
) -> Result<SharedMemoryWithMutex<T>, Box<dyn std::error::Error>> {
    let mutex_size = Mutex::size_of(None);
    let memory = get_shared_memory(is_owner, file_name, mutex_size + std::mem::size_of::<T>())?;
    let is_owner = memory.is_owner();
    let base_ptr = memory.as_ptr();
    let ptr = unsafe { base_ptr.add(Mutex::size_of(Some(base_ptr))) };
    let (mutex, _) = if is_owner {
        unsafe { Mutex::new(base_ptr, ptr)? }
    } else {
        unsafe { Mutex::from_existing(base_ptr, ptr)? }
    };
    Ok(SharedMemoryWithMutex {
        memory,
        memory_type: std::marker::PhantomData,
        mutex,
    })
}

fn get_shared_memory(
    is_owner: bool,
    file_name: &str,
    size: usize,
) -> Result<Shmem, Box<dyn std::error::Error>> {
    Ok(if is_owner {
        ShmemConf::new()
            .size(size)
            .flink(file_name)
            .force_create_flink()
            .create()
    } else {
        ShmemConf::new().flink(file_name).open()
    }?)
    // // The following code doesn't need to know who is first, but if owner quits without
    // // deleting the flink it will panic!
    // Ok(
    //     match ShmemConf::new().size(size).flink(file_name).create() {
    //         Ok(m) => m,
    //         Err(ShmemError::LinkExists) => ShmemConf::new().flink(file_name).open()?,
    //         Err(e) => return Err(Box::new(e)),
    //     },
    // )
}
