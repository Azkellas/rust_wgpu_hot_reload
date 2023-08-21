use rust_embed::RustEmbed;

/// Shader helpers
/// Will load from file in native debug mode to allow reloading at runtime
/// and embed in binary in wasm/release mode.
#[derive(RustEmbed)]
#[folder = "../shaders/"]
pub struct ShaderBuilder;

impl ShaderBuilder {
    /// Load a shader file. Will crash if the file does not exist.
    /// Does not do any pre-processing here, but returns the raw content.
    pub fn load(name: &str) -> String {
        std::str::from_utf8(
            Self::get(name)
                .unwrap_or_else(|| panic!("Could not load shader file: {name}"))
                .data
                .as_ref(),
        )
        .expect("Shader file is not a valid utf8.")
        .to_owned()
    }

    /// Build a shader file by importing all its dependencies.
    /// todo: Add #ifdef #else #endif #ifndef support.
    pub fn build(name: &str) -> String {
        Self::build_with_seen(name, &mut vec![])
    }

    /// Build a shader file by importing all its dependencies.
    /// We use seen to make sure we do not import the same file twice.
    /// Order of import does not matter in wgsl, as it does not in rust
    /// so we don't need to sort the imports depending on their dependencies.
    /// However we cannot define the same symbol twice, so we need to make sure
    /// we do not import the same file twice.
    fn build_with_seen(name: &str, seen: &mut Vec<String>) -> String {
        if seen.contains(&name.to_owned()) {
            return "".to_owned();
        }

        seen.push(name.to_owned());

        Self::load(name)
            .lines()
            .map(|line| {
                // example of valid import: #import "common.wgsl"
                // note: this follow the bevy preprocessor syntax.
                // wgsl-analyzer is also based on the bevy preprocessor.
                // but does not support #import "file" as of August 2023.
                if line.starts_with("#import") {
                    let include = line
                        .split('"')
                        .nth(1)
                        .expect("Invalid import syntax: expected #import \"file\"");
                    let include_content = Self::build_with_seen(include, seen);
                    // We keep but comment the import for debugging purposes.
                    format!("//{line}\n {include_content}")
                } else {
                    line.to_owned() + "\n"
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    #[ignore] // this test require a gpu, ignored by default since it is slow and github actions do not provide a gpu.
    fn test_shader_builder() {
        // build shader.
        let shader = ShaderBuilder::build("test_preprocessor/draw.wgsl");

        // make sure it has everything required.
        assert!(shader.contains("@vertex"));
        assert!(shader.contains("@fragment"));
        assert!(shader.contains("@group(0) @binding(0)"));

        // make sure it compiles.
        // note: heavy setup, does wgpu provide a simpler way to test?
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(wgpu::util::initialize_adapter_from_env_or_default(
            &instance, None,
        ))
        .unwrap();

        let (device, _) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits:
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            },
            None,
        ))
        .unwrap();

        device.push_error_scope(wgpu::ErrorFilter::Validation);

        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader.as_str())),
        });

        // Make sure the compilation didn't return any error.
        assert!(pollster::block_on(device.pop_error_scope()).is_none());
    }
}
