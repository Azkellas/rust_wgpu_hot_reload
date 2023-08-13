use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../shaders/"]
pub struct Shader;

impl Shader {
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

#[derive(PartialEq, Eq)]
pub enum LibState {
    Stable,
    Reloading,
    Reloaded,
}
pub struct ReloadFlags {
    pub shaders: Vec<String>,
    pub lib: LibState,
}
