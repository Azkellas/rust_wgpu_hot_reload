// Flocking boids example with gpu compute update pass
// adapted from https://github.com/gfx-rs/wgpu/tree/trunk/examples/boids

// This example cannot run in WebGL because it uses compute shaders.
// See the README for more details.

use nanorand::{Rng, WyRand};
use wgpu::util::DeviceExt;

use crate::frame_rate::FrameRate;
use crate::program::{Program, ProgramError};
use crate::shader_builder::ShaderBuilder;

const NUM_PARTICLES: u32 = 1500;
const PARTICLES_PER_GROUP: u32 = 64;

struct ComputePass {
    compute_pipeline: wgpu::ComputePipeline,
    particle_bind_groups: Vec<wgpu::BindGroup>,
    work_group_count: u32,
    parameters: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
}

struct RenderPass {
    render_pipeline: wgpu::RenderPipeline,
    particle_buffers: Vec<wgpu::Buffer>,
    vertices_buffer: wgpu::Buffer,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DemoBoidsSettings {
    delta_t: f32,        // cohesion
    rule1_distance: f32, // separation
    rule2_distance: f32, // alignment
    rule3_distance: f32,
    rule1_scale: f32,
    rule2_scale: f32,
    rule3_scale: f32,
    speed: f32,
}

impl DemoBoidsSettings {
    pub fn new() -> Self {
        Self {
            delta_t: 0.04f32,
            rule1_distance: 0.08,
            rule2_distance: 0.025,
            rule3_distance: 0.025,
            rule1_scale: 0.02,
            rule2_scale: 0.05,
            rule3_scale: 0.005,
            speed: 1.0,
        }
    }

    pub fn get_size() -> u64 {
        std::mem::size_of::<Self>() as _
    }
}

/// Example struct holds references to wgpu resources and frame persistent data
pub struct DemoBoidsProgram {
    settings: DemoBoidsSettings,
    compute_pass: ComputePass,
    render_pass: RenderPass,
    frame_rate: FrameRate,
    last_update: web_time::Instant,
}

impl Program for DemoBoidsProgram {
    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::COMPUTE_SHADERS,
            ..Default::default()
        }
    }

    fn required_limits() -> wgpu::Limits {
        // Stricter than default.
        wgpu::Limits::downlevel_defaults()
    }

    /// Get program name.
    fn get_name() -> &'static str {
        "Demo boids"
    }

    /// constructs initial instance of Example struct
    fn init(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
        _surface_configuration: &wgpu::SurfaceConfiguration,
    ) -> Result<Self, ProgramError> {
        let settings = DemoBoidsSettings::new();

        let (compute_pass, render_pass) = Self::create_passes(surface, device, adapter)?;

        Ok(DemoBoidsProgram {
            settings,
            compute_pass,
            render_pass,
            frame_rate: FrameRate::new(100),
            last_update: web_time::Instant::now(),
        })
    }

    /// update is called for any WindowEvent not handled by the framework
    fn update_passes(
        &mut self,
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<(), ProgramError> {
        self.compute_pass.compute_pipeline =
            Self::create_compute_pipeline(device, &self.compute_pass.bind_group_layout)?;
        self.render_pass.render_pipeline = Self::create_render_pipeline(surface, device, adapter)?;

        Ok(())
    }

    /// resize is called on WindowEvent::Resized events
    fn resize(
        &mut self,
        _surface_configuration: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }

    fn update(&mut self, queue: &wgpu::Queue) {
        let last_frame_duration = self.last_update.elapsed().as_secs_f32();
        self.frame_rate.update(last_frame_duration);
        self.last_update = web_time::Instant::now();

        // update speed from rust only for demo purposes.
        self.settings.speed = 1.0;

        // update simulation parameters on gpu.
        self.settings.delta_t = last_frame_duration;
        queue.write_buffer(
            &self.compute_pass.parameters,
            0,
            bytemuck::cast_slice(&[self.settings]),
        );
    }

    /// Draw ui with egui.
    fn draw_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Settings");
        ui.separator();
        ui.label("Cohesion");
        ui.add(egui::Slider::new(&mut self.settings.rule1_distance, 0.0..=0.1).text("distance"));
        ui.add(egui::Slider::new(&mut self.settings.rule1_scale, 0.0..=0.1).text("scale"));

        ui.separator();

        ui.label("Separation");
        ui.add(egui::Slider::new(&mut self.settings.rule2_distance, 0.0..=0.1).text("distance"));
        ui.add(egui::Slider::new(&mut self.settings.rule2_scale, 0.0..=0.1).text("scale"));

        ui.separator();

        ui.label("Alignment");
        ui.add(egui::Slider::new(&mut self.settings.rule3_distance, 0.0..=0.1).text("distance"));
        ui.add(egui::Slider::new(&mut self.settings.rule3_scale, 0.0..=0.1).text("scale"));

        ui.separator();

        ui.label(std::format!(
            "speed: {} (rust only for demo purposes)",
            self.settings.speed
        ));
        ui.label(std::format!("framerate: {:.0}fps", self.frame_rate.get()));
    }

    /// render is called each frame, dispatching compute groups proportional
    ///   a TriangleList draw call for all NUM_PARTICLES at 3 vertices each
    fn render(&self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        // create render pass descriptor and its color attachments
        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })];
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        // get command encoder
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        command_encoder.push_debug_group("compute boid movement");
        {
            // compute pass
            let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.compute_pass.compute_pipeline);
            cpass.set_bind_group(
                0,
                &self.compute_pass.particle_bind_groups[self.frame_rate.get_parity() as usize],
                &[],
            );
            cpass.dispatch_workgroups(self.compute_pass.work_group_count, 1, 1);
        }
        command_encoder.pop_debug_group();

        command_encoder.push_debug_group("render boids");
        {
            // render pass
            let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            rpass.set_pipeline(&self.render_pass.render_pipeline);
            // render dst particles
            rpass.set_vertex_buffer(
                0,
                self.render_pass.particle_buffers[(self.frame_rate.get_parity() as usize + 1) % 2]
                    .slice(..),
            );
            // the three instance-local vertices
            rpass.set_vertex_buffer(1, self.render_pass.vertices_buffer.slice(..));
            rpass.draw(0..3, 0..NUM_PARTICLES);
        }
        command_encoder.pop_debug_group();

        // done
        queue.submit(Some(command_encoder.finish()));
    }
}

impl DemoBoidsProgram {
    fn create_compute_pipeline(
        device: &wgpu::Device,
        compute_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Result<wgpu::ComputePipeline, ProgramError> {
        let compute_shader = ShaderBuilder::create_module(device, "demo_boids/compute.wgsl")?;

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute"),
                bind_group_layouts: &[compute_bind_group_layout],
                push_constant_ranges: &[],
            });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        });

        Ok(compute_pipeline)
    }

    fn create_render_pipeline(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<wgpu::RenderPipeline, ProgramError> {
        let draw_shader = ShaderBuilder::create_module(device, "demo_boids/draw.wgsl")?;

        let swapchain_capabilities = surface.get_capabilities(adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &draw_shader,
                entry_point: "main_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 4,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                    },
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &draw_shader,
                entry_point: "main_fs",
                targets: &[Some(swapchain_format.into())],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Ok(render_pipeline)
    }

    fn create_passes(
        surface: &wgpu::Surface,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) -> Result<(ComputePass, RenderPass), ProgramError> {
        let sim_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Simulation Parameter Buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: DemoBoidsSettings::get_size(),
            mapped_at_creation: false,
        });

        let vertex_buffer_data = [-0.01f32, -0.02, 0.01, -0.02, 0.00, 0.02];
        let vertices_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&vertex_buffer_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let mut initial_particle_data = vec![0.0f32; (4 * NUM_PARTICLES) as usize];
        let mut rng = WyRand::new_seed(42);
        let mut unif = || rng.generate::<f32>() * 2f32 - 1f32; // Generate a num (-1, 1)
        for particle_instance_chunk in initial_particle_data.chunks_mut(4) {
            particle_instance_chunk[0] = unif(); // posx
            particle_instance_chunk[1] = unif(); // posy
            particle_instance_chunk[2] = unif() * 0.1; // velx
            particle_instance_chunk[3] = unif() * 0.1; // vely
        }

        // creates two buffers of particle data each of size NUM_PARTICLES
        // the two buffers alternate as dst and src for each frame

        let mut particle_buffers = Vec::<wgpu::Buffer>::new();

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(DemoBoidsSettings::get_size()),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new((NUM_PARTICLES * 16) as _),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new((NUM_PARTICLES * 16) as _),
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let mut particle_bind_groups = Vec::<wgpu::BindGroup>::new();
        for i in 0..2 {
            particle_buffers.push(
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("Particle Buffer {i}")),
                    contents: bytemuck::cast_slice(&initial_particle_data),
                    usage: wgpu::BufferUsages::VERTEX
                        | wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST,
                }),
            );
        }

        // create two bind groups, one for each buffer as the src
        // where the alternate buffer is used as the dst

        for i in 0..2 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: sim_param_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: particle_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                    },
                ],
                label: None,
            }));
        }

        for i in 0..2 {
            particle_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: sim_param_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: particle_buffers[i].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: particle_buffers[(i + 1) % 2].as_entire_binding(), // bind to opposite buffer
                    },
                ],
                label: None,
            }));
        }

        // calculates number of work groups from PARTICLES_PER_GROUP constant
        let work_group_count =
            ((NUM_PARTICLES as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let compute_pipeline = Self::create_compute_pipeline(device, &compute_bind_group_layout)?;
        let render_pipeline = Self::create_render_pipeline(surface, device, adapter)?;

        Ok((
            ComputePass {
                compute_pipeline,
                particle_bind_groups,
                work_group_count,
                parameters: sim_param_buffer,
                bind_group_layout: compute_bind_group_layout,
            },
            RenderPass {
                render_pipeline,
                particle_buffers,
                vertices_buffer,
            },
        ))
    }
}
