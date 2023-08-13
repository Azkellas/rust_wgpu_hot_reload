use std::borrow::Cow;

use crate::helpers::Shader;

use crate::program::{Program, ProgramError};

pub struct DemoProgram {
    render_pipeline: wgpu::RenderPipeline,
}

impl Program for DemoProgram {
    fn init(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<Self, ProgramError> {
        let render_pipeline = Self::create_render_pipeline(surface, device, adapter)?;

        Ok(Self { render_pipeline })
    }

    /// Create render pipeline.
    fn update_render_pipeline(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<(), ProgramError> {
        self.render_pipeline = Self::create_render_pipeline(surface, device, adapter)?;
        Ok(())
    }

    fn resize(
        &mut self,
        _surface_configuration: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }

    /// Update program before rendering: nothing to do in our case.
    fn update(&mut self) {}

    /// Render program.
    fn render(&self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

impl DemoProgram {
    /// Create render pipeline.
    fn create_render_pipeline(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<wgpu::RenderPipeline, ProgramError> {
        let shader = Shader::load("draw.wgsl");
        #[cfg(not(target_arch = "wasm32"))]
        #[cfg(debug_assertions)]
        {
            // in reload mode, we need to parse the shader to check for errors
            // since wgpu does not return errors when creating the shader module
            // but instantly crash.
            // this means in reload/debug mode, we parse the shader twice.
            let mut frontend = naga::front::wgsl::Frontend::new();
            frontend
                .parse(shader.as_str())
                .map_err(|e| ProgramError::ShaderParseError(e.message().into()))?;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader.as_str())),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(pipeline)
    }
}
