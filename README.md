# Rust WGPU hot-reload

https://github.com/Azkellas/rust_wgpu_hot_reload/assets/29731210/49643bd5-b56c-460e-940f-13d498e1533e

Hot reload both shaders and rust code while developing a WGPU application with [egui](https://github.com/emilk/egui) integration.

Package a single binary/wasm for native and web targets.

---

### Features

- build to native and wasm
- hot reload shader (instant), embed shader files in release and wasm builds
- hot reload rust (~4sec to rebuild and relink library)
- hot reload ui with [egui](https://github.com/emilk/egui) integration
- shader preprocessor
  - `#import "file.wgsl"`

---

### Running the project

To have both shader and rust hot reload, run:
  - `cargo run --features reload` in one terminal
  - `cargo watch -w lib -x "rustc --crate-type=dylib -p lib"` in another terminal. (requires [cargo-watch](https://github.com/watchexec/cargo-watch))

Alternatively use `cargo runcc -c runcc.yml` to run both commands at the same time. (requires [cargo-runcc](https://crates.io/crates/runcc))

You can also run `cargo run` if you only care about shader hot reloading. 

`cargo run --release` as usual to build a single executable for your native target. For `wasm` builds, see below.

---

### Building for the web

```sh
# First install the version of wasm-bindgen-cli that matches the version used by wgpu:
cargo install -f wasm-bindgen-cli --version 0.2.87

# If needed, add the wasm32 target toolchain
rustup target add wasm32-unknown-unknown

# Then build the demo for `wasm32-unknown-unknown`
cargo build --target wasm32-unknown-unknown

# And generate wasm bindings:
wasm-bindgen --out-dir target/generated --web target/wasm32-unknown-unknown/debug/wgpu-hot-reload.wasm

# Copy the index.html in `target/generated`
cp index.html target/generated
```

Lastly, run a web server locally inside the `target/generated` directory to see the project in the browser.
Examples of servers are rust's
[`simple-http-server target/generated`](https://crates.io/crates/simple-http-server) and 
[`miniserve target/generated`](https://crates.io/crates/miniserve).


> Note: as of 2023-08-14, wegbpu is available in chromium-based browsers and in firefox nightly behind a flag. See [webgpu.io](https://webgpu.io) for up to date browser implementation status. If WebGL2 is not enough and you need WebGPU, you need to add the flag `RUSTFLAGS=--cfg=web_sys_unstable_apis` before compiling.

---

### Wgsl preprocessing

This project contains a small homemade preprocessor for wgsl files.
It currently allows to include other files by using `#import "path/to/file.wgsl"` in your shader files.

This syntax follows the bevy preprocessor syntax, which is roughly supported by wgsl-analyzer.

--- 

### Using the template

The project comes with a `Program` trait. Hopefully it should be enough for your needs. You just need to replace the current implementation with yours in `lib/lib.rs`: `pub use crate::demo::DemoProgram as CurrentProgram;`.


---

### Project architecture

The project is divided in two crates: `lib` and `src`.

`src` should only contain the minimal code necessary to start the application and the windowing system,
allowing a maximum of code to be hot-reloaded in `lib` which is built as a dynamic library and reloaded at runtime whenever changes are saved.

Entry point functions should be written in `lib/src/lib.rs`, be public and have the `#[no_mangle]` attribute to be hot-reloadable.
Note that they cannot be generic. Example:

```rust
#[no_mangle]
pub fn get_program_name(program: &CurrentProgram) -> String {
    program.get_name().to_owned()
}
```

Then each use of the `lib` in `src` should be done via the module `hot_lib::library_bridge` that makes the bridge between the binary and the dynamic library.

See [hot-lib-reloader-rs](https://github.com/rksm/hot-lib-reloader-rs) for more information about hot reloading Rust code and its limitations.


---

### Troubleshooting

- Since the wasm and native targets use different flags, switching from one target to the other takes time as many dependencies need to be rebuilt.
Be careful to set rust-analyzer to the same target you're building to, otherwise they will compete against each other and create a lot of unnecessary recompiling.

- Rust requires the dylib to be present at compile time to link, so starting the hot-reload mode with runcc can crash if the bin finishes compiling when the no dll/so is present yet.
In which case you just have to let the library finish building in dynamic mode and restart runcc.

- wgpu does not use the idiomatic rust way `Error` to handle errors. See [here](https://github.com/gfx-rs/wgpu/issues/3767) for more info, or have a look at `shader_build.rs::tests::test_shader_builder` for an example.

---

### References:

- [hot-lib-reloader-rs](https://github.com/rksm/hot-lib-reloader-rs)
- [wgpu](https://github.com/gfx-rs/wgpu) and its [boids](https://github.com/gfx-rs/wgpu/tree/trunk/examples/boids) example
- [learn-wgpu](https://sotrh.github.io/learn-wgpu/)
- [egui](https://github.com/emilk/egui)
