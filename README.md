# Hot Reload Example in Rust

A different take from https://github.com/irh/rust-hot-reloading and https://fasterthanli.me/articles/so-you-want-to-live-reload-rust on hot reloading in Rust. Instead of dynamically loaded libraries I use shared memory to communicate between the process running an OS window and the process that renders it.

To run it, use:

```bash
cargo run &; cargo-watch -s "cargo run -p example-impl"
```
