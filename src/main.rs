//! wgpu hot reload template.
//! this project is divided in two packages:
//! this one with static code
//! and lib that contains hot-reloadable code.
//!
//! `src` is in charge of watching the code to be updated and
//! running the main thread. Almost nothing should be done in this package.
//!
//! `lib` should contain all project-specific code.
//! See `lib::program::Program` for the trait to implement
//! and `lib::demo::DemoProgram` for an example.

mod hot_lib;
mod runner;

use std::sync::{Arc, Mutex};

#[cfg(feature = "reload")]
use crate::hot_lib::library_bridge;

#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
use std::path::Path;

/// Watch shader folder. Only done in native debug mode.
/// Everytime a shader is modified/added/deleted,
/// it will update the `ReloadFlags` so the program can reload them.
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
fn watch<P: AsRef<Path>>(
    path: P,
    data: Arc<Mutex<lib::reload_flags::ReloadFlags>>,
) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                log::info!("Change: {:?}", event.paths);
                let mut data = data.lock().unwrap();
                event.paths.iter().for_each(|p| {
                    let shader_path = p.to_str().unwrap().to_owned();
                    data.shaders.push(shader_path);
                });
            }
            Err(error) => log::error!("Error: {error:?}"),
        }
    }

    Ok(())
}

/// App entry point.
fn main() {
    let data = Arc::new(Mutex::new(lib::reload_flags::ReloadFlags {
        shaders: vec![],
        lib: lib::reload_flags::LibState::Stable,
    }));

    #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
    {
        // Watch shaders folder.
        // When a shader is saved, the pipeline will be recreated.
        let path = "shaders";
        log::info!("Watching {path}");
        let data = data.clone();
        std::thread::spawn(move || {
            if let Err(error) = watch(path, data) {
                log::error!("Could not watch shaders folder: {error:?}");
            }
        });
    }

    #[cfg(feature = "reload")]
    {
        // dll watcher.
        let data = data.clone();
        std::thread::spawn(move || loop {
            // wait until a reload is ready.
            {
                library_bridge::subscribe().wait_for_about_to_reload();
                // update lib state to reloading.
                let mut data = data.lock().unwrap();
                data.lib = lib::reload_flags::LibState::Reloading;
            }

            // allow reload.
            {
                library_bridge::subscribe().wait_for_reload();
                // update lib state to reloaded.
                let mut data = data.lock().unwrap();
                data.lib = lib::reload_flags::LibState::Reloaded;
                log::info!(
                    "Rust lib has been reloaded {} times",
                    library_bridge::version()
                );
            }
        });
    }

    runner::start_app(data);
}
