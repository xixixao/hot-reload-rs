# Hot Reload Example in Rust

A different take from https://github.com/irh/rust-hot-reloading and https://fasterthanli.me/articles/so-you-want-to-live-reload-rust on hot reloading in Rust. Instead of dynamically loaded libraries I use shared memory to communicate between the process running an OS window and the process that renders it.

To run it, use:

```fish
cargo run &; RUST_BACKTRACE=1 cargo-watch -s "cargo run -p example-impl"
```

## Why hot-reload?

The focus of this repo is on graphical applications. For any graphical application it is useful to be able to change what the application displays without the needed to restart it.

## Tutorial

Starting from a vanilla mini-fb application, we convert it to a hot-reload-capable version.

### 1. Decide what state should be persisted and shared

When hot-reloading we will have two running processes:

- **owner**, which owns the window, the graphics integration with the OS
- **reloadable**, which we want to be able to amend to render something else
  Either process can contain state, but since the **reloadable** will be restarted when we make changes, any state it owns will be lost.

At minimum, for minifb, we will want to have a shared `buffer` which the **reloadable** process will render into.

Other examples of shared state are:

- The window size, so that we don't need to hardcode or pass the window size to both processes
- User input, like clicks, can be passed from **owner** to **reloadable**

### 2. Create shared state definition

Using the `HotReload` library we will declare

### 3. Split up the implementation

Instead of

buffer = vec![];
render(buffer)
update_window(buffer)

Do

HotReloadable {buffer} = reloadable
update_window(buffer)
