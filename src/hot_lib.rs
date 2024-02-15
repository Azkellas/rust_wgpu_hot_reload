/// Library bridge when hot reloading.
#[cfg(feature = "reload")]
#[hot_lib_reloader::hot_module(dylib = "lib")]
pub mod library_bridge {
    // Any type that is used in the functions signatures in lib.rs should be imported here.
    pub use lib::program::{Program, ProgramError};
    pub use lib::reload_flags::{LibState, ReloadFlags};
    pub use lib::CameraLookAt;
    pub use lib::CurrentProgram;

    // Specific hot reload helpers.
    hot_functions_from_file!("lib/src/lib.rs");

    // expose a type to subscribe to lib load events
    #[lib_change_subscription]
    pub fn subscribe() -> hot_lib_reloader::LibReloadObserver {}

    // a monotonically increasing counter (starting with 0) that counts library reloads
    #[lib_version]
    pub fn version() -> usize {}
}

/// Library bridge when rust reload is disabled.
#[cfg(not(feature = "reload"))]
pub mod library_bridge {
    // pub use lib::program::{Program, ProgramError};
    pub use lib::reload_flags::{LibState, ReloadFlags};
    // pub use lib::CurrentProgram;

    // Include lib file directly since it is not done via the hot-reload module.
    pub use lib::*;
}
