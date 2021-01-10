use hot_reload::simple_shared_memory::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: Example shows the problem when two threads wait at the same time
// I don't need them to, but it seems that the way watch kills the child
// process doesn't end the p_wait - I need to find out how the thread
// kill works and how to stop the p_thread_cond_wait, or figure out
// how to share the mutex - although that might not be possible
// across processes
fn main() -> Result<()> {
  let mut channel = shared_channel::<u32>(true, "example")?;

  let child = std::thread::spawn(move || {
    let mut channel = shared_channel::<u32>(false, "example").unwrap();
    println!("\tWaiting for event to be signaled !");
    let _ = channel.recv();
    println!("\tSignaled !");
  });

  let child2 = std::thread::spawn(move || {
    let mut channel = shared_channel::<u32>(false, "example").unwrap();
    println!("\tWaiting for event to be signaled !");
    let _ = channel.recv();
    println!("\tSignaled !");
  });
  println!("Setting event to signaled");
  std::thread::sleep(std::time::Duration::from_secs(3));
  channel.send(&1);
  child.join().unwrap();
  child2.join().unwrap();
  println!("Setting event to signaled");

  channel.send(&1);
  Ok(())
}
