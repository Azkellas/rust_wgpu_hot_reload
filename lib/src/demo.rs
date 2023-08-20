use std::borrow::Cow;

use wgpu::util::DeviceExt;

use crate::frame_rate::FrameRate;
use crate::helpers::Shader;
use crate::pass::Pass;
use crate::program::{Program, ProgramError};

/// Settings for the `DemoProgram`
/// `polygon_edge_count` is not exposed in ui on purpose for demo purposes
/// change it in the code with hot-reload enable to see it working.
#[derive(Clone, Copy, Debug)]
pub struct DemoSettings {
    /// polygon radius in window, between 0 and 1
    polygon_size: f32, // exposed in ui
    /// regular polygon edge count, expected to be 3 or more
    polygon_edge_count: u32, // exposed in rust only
    /// speed of the rotation
    speed: f32, // exposed in ui
}

/// Demo Program rotation a regular polygon showcasing the three type of live updates
///     shader: `draw.wgsl`
///     rust: `polygon_edge_count` in `DemoProgram::update`
///     ui: `size` and `speed`
#[derive(Debug)]
pub struct DemoProgram {
    render_pass: Pass,
    _start_time: instant::Instant, // std::time::Instant is not compatible with wasm
    last_update: instant::Instant,
    settings: DemoSettings,
    elapsed: f32, // elapsed take the speed into consideration
    frame_rate: FrameRate,
}

impl Program for DemoProgram {
    /// Create program.
    /// Assume the `render_pipeline` will be properly initialized.
    fn init(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<Self, ProgramError> {
        let render_pass = Self::create_render_pass(surface, device, adapter)?;

        Ok(Self {
            render_pass,
            _start_time: instant::Instant::now(),
            last_update: instant::Instant::now(),
            settings: DemoSettings {
                polygon_size: 0.5,
                polygon_edge_count: 3,
                speed: 1.0,
            },
            elapsed: 0.0,
            frame_rate: FrameRate::default(),
        })
    }

    /// Get program name.
    fn get_name(&self) -> &'static str {
        "Demo triangle"
    }

    /// Recreate render pass.
    fn update_passes(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<(), ProgramError> {
        self.render_pass = Self::create_render_pass(surface, device, adapter)?;
        Ok(())
    }

    // Resize owned textures if needed, nothing for the demo here.
    fn resize(
        &mut self,
        _surface_configuration: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }

    /// Update program before rendering.
    fn update(&mut self, queue: &wgpu::Queue) {
        // Set the edge count of the regular polygon.
        // This is not exposed in the ui on purpose to demonstrate the rust hot reload.
        self.settings.polygon_edge_count = 150;

        // update elapsed time, taking speed into consideration.
        let last_frame_duration = self.last_update.elapsed().as_secs_f32();
        self.elapsed += last_frame_duration * self.settings.speed;
        self.frame_rate.update(last_frame_duration);
        self.last_update = instant::Instant::now();
        queue.write_buffer(
            &self.render_pass.uniform_buf,
            0,
            bytemuck::cast_slice(&[
                self.elapsed,
                self.settings.polygon_size,
                self.settings.polygon_edge_count as f32,
            ]),
        );
    }

    /// Render program.
    fn render(&self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        // We draw a regular polygon with n edges
        // by drawing the n triangles starting from the center and with two adjacent vertices
        // hence the * 3 vertex count, a square results in 4 triangles so 12 vertices to draw.
        let vertex_count = self.settings.polygon_edge_count * 3;

        // Create a command encoder.
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            // render pass.
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            render_pass.set_pipeline(&self.render_pass.pipeline);
            render_pass.set_bind_group(0, &self.render_pass.bind_group, &[]);
            render_pass.draw(0..vertex_count, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }

    /// Draw ui with egui.
    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();
        ui.add(egui::Slider::new(&mut self.settings.polygon_size, 0.0..=1.0).text("size"));
        ui.add(egui::Slider::new(&mut self.settings.speed, 0.0..=20.0).text("speed"));
        ui.separator();
        ui.label(std::format!("framerate: {:.0}fps", self.frame_rate.get()));
    }
}

impl DemoProgram {
    /// Create render pipeline.
    /// In debug mode it will return a `ProgramError` if it failed compiling a shader
    /// In release/wasm, il will crash since wgpu does not return errors in such situations.
    fn create_render_pipeline(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        uniforms_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<wgpu::RenderPipeline, ProgramError> {
        let shader_path = "draw.wgsl";
        let shader = Shader::load(shader_path);

        // device.create_shader_module panics if the shader is malformed
        // only check this on native debug builds.
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        device.push_error_scope(wgpu::ErrorFilter::Validation);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("draw.wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader.as_str())),
        });

        // device.create_shader_module panics if the shader is malformed
        // only check this on native debug builds.
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        if let Some(error) = pollster::block_on(device.pop_error_scope()) {
            log::error!("{}", error);
            // redundant, naga already logs the error.
            return Err(ProgramError::ShaderParseError(format!(
                "{}: {}",
                shader_path, error
            )));
        }

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

    /// Create render pass.
    /// Will return an error in debug, and crash in release/wasm if a shader is malformed.
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
