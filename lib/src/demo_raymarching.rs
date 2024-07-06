use wgpu::util::DeviceExt;

use crate::camera_control::CameraLookAt;
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
    // Index buffer.
    pub index_buffer: wgpu::Buffer,
    // Vertex buffer.
    pub vertex_buffer: wgpu::Buffer,
    //
    pub index_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

// lib.rs
impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DemoRaymarchingSettings {
    pub camera: CameraLookAt,
    pub size: [f32; 2],
    pub elapsed: f32,   // elapsed take the speed into consideration
    _padding: [f32; 2], // padding for alignment
}

/// Demo raymarching program.
/// Everything is done in the shader.
/// Provides both 2d and 3d raymarching.
#[derive(Debug)]
pub struct DemoRaymarchingProgram {
    render_pass: Pass,
    _start_time: web_time::Instant, // std::time::Instant is not compatible with wasm
    last_update: web_time::Instant,
    frame_rate: FrameRate,
    settings: DemoRaymarchingSettings,
}

impl DemoRaymarchingSettings {
    pub fn new(surface_configuration: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            camera: CameraLookAt::default(),
            elapsed: 0.0,
            size: [
                surface_configuration.width as f32,
                surface_configuration.height as f32,
            ],
            _padding: [0.0; 2],
        }
    }

    pub fn get_size() -> u64 {
        std::mem::size_of::<Self>() as _
    }
}

impl Program for DemoRaymarchingProgram {
    /// Create program.
    /// Assume the `render_pipeline` will be properly initialized.
    fn init(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, ProgramError> {
        let render_pass = Self::create_render_pass(surface, device, adapter)?;

        Ok(Self {
            render_pass,
            _start_time: web_time::Instant::now(),
            last_update: web_time::Instant::now(),
            frame_rate: FrameRate::new(100),
            settings: DemoRaymarchingSettings::new(surface_configuration),
        })
    }

    /// Get program name.
    fn get_name() -> &'static str {
        "Demo raymarching"
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
        surface_configuration: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.settings.size[0] = surface_configuration.width as f32;
        self.settings.size[1] = surface_configuration.height as f32;
    }

    /// Update program before rendering.
    fn update(&mut self, queue: &wgpu::Queue) {
        // Set the edge count of the regular raymarching.
        // This is not exposed in the ui on purpose to demonstrate the rust hot reload.

        // update elapsed time, taking speed into consideration.
        let last_frame_duration = self.last_update.elapsed().as_secs_f32();
        self.settings.elapsed += last_frame_duration;
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
            render_pass.set_vertex_buffer(0, self.render_pass.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.render_pass.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            ); // 1.
            render_pass.draw_indexed(0..self.render_pass.index_count, 0, 0..1); // 2.
        }

        queue.submit(Some(encoder.finish()));
    }

    /// Draw ui with egui.
    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();
        ui.label(std::format!("framerate: {:.0}fps", self.frame_rate.get()));
    }

    fn get_camera(&mut self) -> Option<&mut crate::camera_control::CameraLookAt> {
        Some(&mut self.settings.camera)
    }
}

impl DemoRaymarchingProgram {
    /// Create render pipeline.
    /// In debug mode it will return a `ProgramError` if it failed compiling a shader
    /// In release/wasm, il will crash since wgpu does not return errors in such situations.
    fn create_render_pipeline(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        uniforms_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<wgpu::RenderPipeline, ProgramError> {
        let shader = ShaderBuilder::create_module(device, "demo_raymarching/draw.wgsl")?;
        // let shader = ShaderBuilder::create_module(device, "test_preprocessor/draw.wgsl")?; // uncomment to test preprocessor

        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[uniforms_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Raymarching Render Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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
            label: Some("Camera Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: DemoRaymarchingSettings::get_size(),
            mapped_at_creation: false,
        });

        let uniforms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
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

        // lib.rs
        const VERTICES: &[Vertex] = &[
            Vertex {
                position: [-1.0, -1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
            },
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        const INDICES: &[u16] = &[1, 0, 2, 2, 0, 3];
        let index_count = INDICES.len() as u32;

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let pipeline =
            Self::create_render_pipeline(surface, device, adapter, &uniforms_bind_group_layout)?;

        Ok(Pass {
            pipeline,
            bind_group: uniforms_bind_group,
            uniform_buf: uniforms,
            index_buffer,
            vertex_buffer,
            index_count,
        })
    }
}
