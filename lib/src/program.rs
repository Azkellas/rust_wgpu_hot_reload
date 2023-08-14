#[derive(Debug)]
pub enum ProgramError {
    ShaderParseError(String),
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
    fn update_render_pipeline(
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
    fn update(&mut self);

    /// Render program.
    fn render<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b;

    fn draw_ui(&mut self, ui: &mut egui::Ui);
}
