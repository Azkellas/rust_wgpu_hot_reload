use std::fmt;

/// Errors a program can return
pub enum ProgramError {
    /// This encapsulate naga::front::wgsl::ParseError that is not available in wasm it seems.
    /// The output is the same minus the colors.
    ShaderParseError(String),
    ShaderNotFound(String),
}

impl fmt::Display for ProgramError {
    /// Display error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShaderParseError(message) => {
                writeln!(f, "Shader parse Error:")?;
                writeln!(f, "{message}")?;
            }
            Self::ShaderNotFound(message) => {
                writeln!(f, "Shader not found: {message}")?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for ProgramError {
    /// Force Debug to be multilined like Display for the sake of clarity in shader files.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

/// Program trait.
///
/// All programs (ie specific projects) should implement this trait.
pub trait Program: Sized {
    /// Create program.
    ///
    /// # Errors
    /// - `ProgramError::ShaderParseError` when the shader could not be compiled.
    fn init(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, ProgramError>;

    /// Get program name.
    fn get_name() -> &'static str;

    /// Create render pipeline.
    ///
    /// # Errors
    /// - `ProgramError::ShaderParseError` when the shader could not be compiled.
    fn update_passes(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<(), ProgramError>;

    /// Resize output
    fn resize(
        &mut self,
        surface_configuration: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    );

    /// Update program before rendering.
    fn update(&mut self, queue: &wgpu::Queue);

    /// Render program.
    fn render(&self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue);

    /// Draw ui.
    fn draw_ui(&mut self, ui: &mut egui::Ui);

    /// Process input. Return true if the input was processed.
    fn process_input(&mut self, _input: &winit_input_helper::WinitInputHelper) -> bool {
        false
    }

    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }
    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::empty(),
            shader_model: wgpu::ShaderModel::Sm5,
            ..wgpu::DownlevelCapabilities::default()
        }
    }
    fn required_limits() -> wgpu::Limits {
        // These downlevel limits will allow the code to run on all possible hardware
        wgpu::Limits::downlevel_webgl2_defaults()
    }

    fn get_camera(&mut self) -> Option<&mut crate::camera_control::CameraLookAt> {
        None
    }
}
