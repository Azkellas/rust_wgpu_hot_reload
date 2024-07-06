use crate::frame_rate::FrameRate;
use crate::program::{Program, ProgramError};
use crate::shader_builder::ShaderBuilder;

/// A simple struct to store a wgpu pass with a uniform buffer.
#[derive(Debug)]
pub struct Pass {
    /// Pipeline that will be called to render the pass
    pub pipeline: wgpu::RenderPipeline,
    /// Buffer bind group for this pass.
    pub bind_group: wgpu::BindGroup,
    /// Single uniform buffer for this pass.
    pub uniform_buf: wgpu::Buffer,
}

/// Settings for the `DemoProgram`
/// `polygon_edge_count` is not exposed in ui on purpose for demo purposes
/// change it in the code with hot-reload enable to see it working.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DemoPolygonSettings {
    // elapsed take the speed into consideration
    elapsed: f32,
    /// polygon radius in window, between 0 and 1
    polygon_size: f32, // exposed in ui
    /// regular polygon edge count, expected to be 3 or more
    polygon_edge_count: u32, // exposed in rust only
    /// speed of the rotation
    speed: f32, // exposed in ui
}

impl DemoPolygonSettings {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            polygon_size: 0.5,
            polygon_edge_count: 3,
            speed: 1.0,
        }
    }

    pub fn get_size() -> u64 {
        std::mem::size_of::<Self>() as _
    }
}
/// Demo Program rotation a regular polygon showcasing the three type of live updates
///     shader: `draw.wgsl`
///     rust: `polygon_edge_count` in `DemoProgram::update`
///     ui: `size` and `speed`
#[derive(Debug)]
pub struct DemoPolygonProgram {
    render_pass: Pass,
    _start_time: web_time::Instant, // std::time::Instant is not compatible with wasm
    last_update: web_time::Instant,
    settings: DemoPolygonSettings,
    frame_rate: FrameRate,
}

impl Program for DemoPolygonProgram {
    /// Create program.
    /// Assume the `render_pipeline` will be properly initialized.
    fn init(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        _surface_configuration: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, ProgramError> {
        let render_pass = Self::create_render_pass(surface, device, adapter)?;

        Ok(Self {
            render_pass,
            _start_time: web_time::Instant::now(),
            last_update: web_time::Instant::now(),
            settings: DemoPolygonSettings::new(),
            frame_rate: FrameRate::default(),
        })
    }

    /// Get program name.
    fn get_name() -> &'static str {
        "Demo polygon"
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
        self.settings.polygon_edge_count = 7;

        // update elapsed time, taking speed into consideration.
        let last_frame_duration = self.last_update.elapsed().as_secs_f32();
        self.settings.elapsed += last_frame_duration * self.settings.speed;
        self.frame_rate.update(last_frame_duration);
        self.last_update = web_time::Instant::now();
        queue.write_buffer(
            &self.render_pass.uniform_buf,
            0,
            bytemuck::cast_slice(&[self.settings]),
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
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
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
        ui.label(std::format!(
            "edge count: {} (rust only for demo purposes)",
            self.settings.polygon_edge_count
        ));
        ui.label(std::format!("framerate: {:.0}fps", self.frame_rate.get()));
    }
}

impl DemoPolygonProgram {
    /// Create render pipeline.
    /// In debug mode it will return a `ProgramError` if it failed compiling a shader
    /// In release/wasm, il will crash since wgpu does not return errors in such situations.
    fn create_render_pipeline(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        uniforms_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<wgpu::RenderPipeline, ProgramError> {
        let shader = ShaderBuilder::create_module(device, "demo_polygon/draw.wgsl")?;
        // let shader = ShaderBuilder::create_module(device, "test_preprocessor/draw.wgsl")?; // uncomment to test preprocessor

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
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
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
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniforms Buffer"),
            size: DemoPolygonSettings::get_size(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
