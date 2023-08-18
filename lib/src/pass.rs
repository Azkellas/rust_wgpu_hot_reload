/// A simple struct to store a wgpu pass with a uniform buffer.
#[derive(Debug)]
pub struct Pass {
    /// Pipeline that will be called to render the pass
    //todo: pipeline cannot be a wgpu::ComputePipeline.
    pub pipeline: wgpu::RenderPipeline,
    /// Buffer bind group for this pass.
    pub bind_group: wgpu::BindGroup,
    /// Single uniform buffer for this pass.
    //todo: only one buffer is allowed in this situation.
    pub uniform_buf: wgpu::Buffer,
}
