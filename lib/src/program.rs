use std::fmt;

pub enum ProgramError {
    // This encapsulate naga::front::wgsl::ParseError that is not available in wasm it seems.
    // The output is the same minus the colors.
    ShaderParseError(String),
}

impl fmt::Display for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ShaderParseError(message) => {
                write!(f, "Shader parse Error:\n")?;
                write!(f, "{message}")?;
            }
        }
        Ok(())
    }
}

/// Force Debug to be multilined like Display for the sake of clarity in shader files.
impl fmt::Debug for ProgramError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

pub trait Program: Sized {
    /// Create program.
    fn init(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<Self, ProgramError>;

    /// Get program name.
    fn get_name(&self) -> &'static str;

    /// Create render pipeline.
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
    fn render<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b;

    fn draw_ui(&mut self, ui: &mut egui::Ui);
}
