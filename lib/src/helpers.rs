use rust_embed::RustEmbed;

/// Shader helpers
/// Will load from file in native debug mode to allow reloading at runtime
/// and embed in binary in wasm/release mode.
#[derive(RustEmbed)]
#[folder = "../shaders/"]
pub struct Shader;

impl Shader {
    /// Load a shader file. Will crash if the file does not exist.
    pub fn load(name: &'static str) -> String {
        std::str::from_utf8(
            Self::get(name)
                .expect("Could not load shader file.")
                .data
                .as_ref(),
        )
        .expect("Shader file is not a valid utf8.")
        .to_owned()
    }
}

/// Library state in hot reload mode
#[derive(PartialEq, Eq)]
pub enum LibState {
    /// Library is stable: nothing to do
    Stable,
    /// Library is reloading: avoid calls to its function until it's done
    Reloading,
    /// Library is done reloading
    Reloaded,
}

/// Reload flags contain the state of the library / shader folder
/// `shaders` contains the shaders that were updated until last rebuild
/// `lib` is the state of the library
pub struct ReloadFlags {
    pub shaders: Vec<String>,
    pub lib: LibState,
}
