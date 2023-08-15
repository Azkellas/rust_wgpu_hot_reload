/// A simple struct to store a wgpu pass with a uniform buffer.
pub struct Pass {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buf: wgpu::Buffer,
}
