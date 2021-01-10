use anyhow::*;
use raw_sync::events::*;
use raw_sync::locks::*;
use raw_sync::Timeout;
use shared_memory::*;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

// Provides a cross-process channel with a familiar API, similar to [`std::sync::mpsc::channel`].
pub fn shared_channel<T>(
  is_owner: bool,
  identifier: &str,
) -> Result<SharedChannel<T>, Box<dyn std::error::Error>>
where
  T: Copy,
{
  Ok(SharedChannel {
    memory: shared_memory_with_event_and_mutex::<SharedChannelInternal<T>>(is_owner, identifier)?,
  })
}

pub struct SharedChannel<T>
where
  T: Copy,
{
  memory: SharedMemoryWithEventAndMutex<SharedChannelInternal<T>>,
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
      self.memory.event.set(EventState::Clear).unwrap();
      return Some(value);
    }
    None
  }

  pub fn recv(&mut self) -> T {
    #[cfg(target_family = "unix")]
    {
      let is_being_killed = Arc::new(AtomicBool::new(false));
      let is_being_killed_for_handler = Arc::clone(&is_being_killed);
      let hook_id = unsafe {
        signal_hook::low_level::register(signal_hook::consts::signal::SIGTERM, move || {
          is_being_killed_for_handler.store(true, Ordering::Relaxed);
        })
        .unwrap()
      };
      while matches!(
        self
          .memory
          .event
          .wait_allow_spurious_wake_up(Timeout::Infinite)
          .unwrap(),
        EventState::Clear
      ) {
        if is_being_killed.load(Ordering::Relaxed) {
          signal_hook::low_level::unregister(hook_id);
          std::process::abort();
        }
      }
    }
    #[cfg(not(target_family = "unix"))]
    {
      self.memory.event.wait(Timeout::Infinite).unwrap();
    }
    self.try_recv().unwrap()
  }

  pub fn send(&mut self, data: &T)
  where
    T: Copy,
  {
    let internal = self.memory.get();
    internal.value = Some(*data);
    self.memory.event.set(EventState::Signaled).unwrap();
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
  memory: Shmem,
  memory_type: std::marker::PhantomData<T>,
  pub length: usize,
}

impl<T> SharedMemory<[T]> for SharedMemorySlice<T> {
  fn get(&mut self) -> &mut [T] {
    self.get()
  }
}

impl<T> SharedMemorySlice<T> {
  pub fn get(&mut self) -> &mut [T] {
    unsafe { std::slice::from_raw_parts_mut(&mut *(self.memory.as_ptr() as *mut T), self.length) }
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
pub struct SharedMemoryWithEventAndMutex<T> {
  #[allow(dead_code)]
  memory: Shmem,
  memory_type: std::marker::PhantomData<T>,
  event: Box<dyn EventImpl>,
  mutex: Box<dyn LockImpl>,
}

impl<T> SharedMemory<T> for SharedMemoryWithEventAndMutex<T> {
  fn get(&mut self) -> &mut T {
    let guard = self.mutex.lock().unwrap();
    unsafe { &mut *(*guard as *mut T) }
  }
}

// Provides a shared memory between two processes, without synchronization, with a size known
// at compile time.
pub fn shared_memory<T>(
  is_owner: bool,
  identifier: &str,
) -> Result<SharedMemorySimple<T>, Box<dyn std::error::Error>> {
  Ok(SharedMemorySimple {
    memory: get_shared_memory(is_owner, identifier, std::mem::size_of::<T>())?,
    memory_type: std::marker::PhantomData,
  })
}

// Provides a shared memory between two processes, without synchronization, with dynamic size.
pub fn shared_memory_with_slice<T>(
  is_owner: bool,
  identifier: &str,
  length: usize,
) -> Result<SharedMemorySlice<T>, Box<dyn std::error::Error>> {
  Ok(SharedMemorySlice {
    memory: get_shared_memory(is_owner, identifier, std::mem::size_of::<T>() * length)?,
    memory_type: std::marker::PhantomData,
    length,
  })
}

// Provides a shared memory between two processes, with synchronization.
pub fn shared_memory_with_mutex<T>(
  is_owner: bool,
  identifier: &str,
) -> Result<SharedMemoryWithMutex<T>, Box<dyn std::error::Error>> {
  let mutex_size = Mutex::size_of(None);
  let memory = get_shared_memory(is_owner, identifier, mutex_size + std::mem::size_of::<T>())?;
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

pub fn shared_memory_with_event_and_mutex<T>(
  is_owner: bool,
  identifier: &str,
) -> Result<SharedMemoryWithEventAndMutex<T>, Box<dyn std::error::Error>> {
  let mutex_size = Mutex::size_of(None);
  let event_size = Event::size_of(None);
  let memory = get_shared_memory(
    is_owner,
    identifier,
    event_size + mutex_size + std::mem::size_of::<T>(),
  )?;
  let is_owner = memory.is_owner();
  let base_ptr = memory.as_ptr();

  let (event, event_size) = if is_owner {
    // `true` because we don't support multiple concurrent receivers
    unsafe { Event::new(base_ptr, true) }
  } else {
    unsafe { Event::from_existing(base_ptr) }
  }?;
  let mutex_base_ptr = unsafe { base_ptr.add(event_size) };
  let ptr = unsafe { mutex_base_ptr.add(Mutex::size_of(Some(mutex_base_ptr))) };
  let (mutex, _) = if is_owner {
    unsafe { Mutex::new(base_ptr, ptr)? }
  } else {
    unsafe { Mutex::from_existing(base_ptr, ptr)? }
  };
  Ok(SharedMemoryWithEventAndMutex {
    memory,
    memory_type: std::marker::PhantomData,
    event,
    mutex,
  })
}

fn get_shared_memory(is_owner: bool, identifier: &str, size: usize) -> anyhow::Result<Shmem> {
  if identifier.len() >= 32 {
    return Err(anyhow!(
      "Tried to create shared memory with identifier {}, \
      which is too long (macOS limits to 32 characters)",
      identifier
    ));
  }
  Ok(if is_owner {
    ShmemConf::new()
      .size(size)
      .os_id(identifier)
      .force_create_flink()
      .create()
  } else {
    ShmemConf::new().os_id(identifier).open()
  }?)
  // // The following code doesn't need to know who is first, but if owner quits without
  // // deleting the flink it will panic!
  // Ok(
  //     match ShmemConf::new().size(size).flink(identifier).create() {
  //         Ok(m) => m,
  //         Err(ShmemError::LinkExists) => ShmemConf::new().flink(identifier).open()?,
  //         Err(e) => return Err(Box::new(e)),
  //     },
  // )
}
