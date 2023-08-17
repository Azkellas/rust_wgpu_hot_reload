use std::borrow::Cow;
use std::time;

use wgpu::util::DeviceExt;

use crate::helpers::Shader;
use crate::pass::Pass;
use crate::program::{Program, ProgramError};

pub struct DemoSettings {
    triangle_size: f32,
    triangle_count: u32,
    speed: f32,
}
pub struct DemoProgram {
    render_pass: Pass,
    start_time: time::Instant,
    settings: DemoSettings,
}

impl Program for DemoProgram {
    fn init(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<Self, ProgramError> {
        let render_pass = Self::create_render_pass(surface, device, adapter)?;

        Ok(Self {
            render_pass,
            start_time: time::Instant::now(),
            settings: DemoSettings {
                triangle_size: 0.5,
                triangle_count: 1,
                speed: 1.0,
            },
        })
    }

    /// Get program name.
    fn get_name(&self) -> &'static str {
        "Demo triangle"
    }

    /// Create render pipeline.
    fn update_passes(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<(), ProgramError> {
        self.render_pass = Self::create_render_pass(surface, device, adapter)?;
        Ok(())
    }

    fn resize(
        &mut self,
        _surface_configuration: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }

    /// Update program before rendering.
    fn update(&mut self, queue: &wgpu::Queue) {
        // update triangle size in uniform buffer
        self.settings.triangle_count = 3;
        queue.write_buffer(
            &self.render_pass.uniform_buf,
            0,
            bytemuck::cast_slice(&[
                self.start_time.elapsed().as_secs_f32(),
                self.settings.triangle_size,
                self.settings.triangle_count as f32,
                self.settings.speed,
            ]),
        );
    }

    /// Render program.
    fn render<'a, 'b>(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b,
    {
        render_pass.set_pipeline(&self.render_pass.pipeline);
        render_pass.set_bind_group(0, &self.render_pass.bind_group, &[]);
        render_pass.draw(0..3 * self.settings.triangle_count, 0..1);
    }

    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Simple demo with a triangle.");
        ui.separator();
        ui.heading("Settings");
        // add button
        ui.add(egui::Slider::new(&mut self.settings.triangle_size, 0.0..=1.0).text("size"));
        ui.add(egui::Slider::new(&mut self.settings.speed, 0.0..=5.0).text("speed"));
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
        uniforms_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<wgpu::RenderPipeline, ProgramError> {
        let shader_path = "draw.wgsl";
        let shader = Shader::load(shader_path);
        #[cfg(not(target_arch = "wasm32"))]
        #[cfg(debug_assertions)]
        {
            // in reload mode, we need to parse the shader to check for errors
            // since wgpu does not return errors when creating the shader module
            // but instantly crash.
            // this means in reload/debug mode, we parse the shader twice.
            let mut frontend = naga::front::wgsl::Frontend::new();
            frontend.parse(shader.as_str()).map_err(|e| {
                ProgramError::ShaderParseError(
                    e.emit_to_string_with_path(shader.as_str(), shader_path),
                )
            })?;
        }

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader.as_str())),
        });

        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[uniforms_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
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

    fn create_render_pass(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<Pass, ProgramError> {
        // create uniform buffer.
        let uniforms = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[0.0, 0.0, 0.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniforms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniforms_bind_group_layout"),
            });

        let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniforms_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
            label: Some("uniforms_bind_group"),
        });

        let pipeline =
            Self::create_render_pipeline(surface, device, adapter, &uniforms_bind_group_layout)?;

        Ok(Pass {
            pipeline,
            bind_group: uniforms_bind_group,
            uniform_buf: uniforms,
        })
    }
}
