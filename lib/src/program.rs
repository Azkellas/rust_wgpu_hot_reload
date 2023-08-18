use std::fmt;

/// Errors a program can return
pub enum ProgramError {
    /// This encapsulate naga::front::wgsl::ParseError that is not available in wasm it seems.
    /// The output is the same minus the colors.
    ShaderParseError(String),
}

impl fmt::Display for ProgramError {
    /// Display error.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShaderParseError(message) => {
                writeln!(f, "Shader parse Error:")?;
                writeln!(f, "{message}")?;
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
    ) -> Result<Self, ProgramError>;

    /// Get program name.
    fn get_name(&self) -> &'static str;

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
    fn render<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b;

    /// Draw ui.
    fn draw_ui(&mut self, ui: &mut egui::Ui);
}
