# Rust WGPU hot-reload

![Demo](demo.gif)

Hot reload both shaders and Rust code while developing a WGPU application.

## How to use

The project is divided in two crates: `lib` and `src`. `lib` is built as a dll and reloaded at runtime.
Entry point functions should be written in `lib.rs`, be public and have the `#[no_mangle]` attribute to be hot-reloadable.
Then each include to the lib should be done via the module `hot_lib::library_bridge`.

The project has three different modes:
- `cargo run` will hot-reload shaders but not rust
- `cargo run --features reload` will hot-reload both shaders and rust. To rebuild the lib after each change, use `cargo watch -w lib -x 'build -p lib'` or use [cargo-runcc](https://crates.io/crates/runcc) and `cargo runcc -c runcc.yml` to run both commands at the same time.
- `cargo run --release` will include the shaders in the binary.



## References:
- See [hot-lib-reloader-rs](https://github.com/rksm/hot-lib-reloader-rs) for more information about hot reloading Rust code.
- [wgpu](https://github.com/gfx-rs/wgpu)
