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

    /// Get program name.
    fn get_name(&self) -> &'static str {
        "Demo triangle"
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
    fn render<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw(0..3, 0..1);
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Simple demo with a triangle.");
        ui.separator();
        ui.heading("Settings");
        // add button
        ui.add(egui::Slider::new(&mut 0.0, 0.0..=1.0).text("Length"));
        if ui.button("Example button").clicked() {
            println!("Button clicked.");
        }
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
