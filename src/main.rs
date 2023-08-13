mod hot_lib;
mod runner;

use std::sync::{Arc, Mutex};

#[cfg(feature = "reload")]
use crate::hot_lib::library_bridge;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(debug_assertions)]
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

#[cfg(not(target_arch = "wasm32"))]
#[cfg(debug_assertions)]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(debug_assertions)]
fn watch<P: AsRef<Path>>(
    path: P,
    data: Arc<Mutex<lib::helpers::ReloadFlags>>,
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
                    let path = p.to_str().unwrap().to_owned();
                    data.shaders.push(path);
                });
            }
            Err(error) => log::error!("Error: {error:?}"),
        }
    }

    Ok(())
}

fn main() {
    let data = Arc::new(Mutex::new(lib::helpers::ReloadFlags {
        shaders: vec![],
        lib: lib::helpers::LibState::Stable,
    }));

    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(debug_assertions)]
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
                data.lib = lib::helpers::LibState::Reloading;
            }

            // allow reload.
            {
                library_bridge::subscribe().wait_for_reload();
                // update lib state to reloaded.
                let mut data = data.lock().unwrap();
                data.lib = lib::helpers::LibState::Reloaded;
                println!(
                    "... library has been reloaded {} times",
                    library_bridge::version()
                );
            }
        });
    }

    runner::start_app(data);
}
