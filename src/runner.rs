use egui_wgpu::{Renderer, ScreenDescriptor};
use std::sync::{Arc, Mutex};
use winit::event::StartCause;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use winit_input_helper::WinitInputHelper;

use crate::hot_lib::library_bridge;

struct EventLoopWrapper {
    event_loop: EventLoop<()>,
    window: Arc<Window>,
}

impl EventLoopWrapper {
    pub fn new(title: &str) -> Self {
        let event_loop = EventLoop::new().unwrap();
        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title(title);

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowBuilderExtWebSys;
            let canvas = web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.create_element("canvas").ok())
                .and_then(|canvas| canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                .expect("couldn't create canvas in document");

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| body.append_child(&canvas).ok())
                .expect("couldn't append canvas to document body");

            builder = builder.with_canvas(Some(canvas));
        }

        let window = Arc::new(builder.build(&event_loop).unwrap());

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;

            let get_full_size = || {
                // TODO Not sure how to get scrollbar dims
                let scrollbars = 4.0;
                let win = web_sys::window().unwrap();
                // `inner_width` corresponds to the browser's `self.innerWidth` function, which are in
                // Logical, not Physical, pixels
                winit::dpi::LogicalSize::new(
                    win.inner_width().unwrap().as_f64().unwrap() - scrollbars,
                    win.inner_height().unwrap().as_f64().unwrap() - scrollbars,
                )
            };

            let size = get_full_size();
            let _ = window.request_inner_size(size);

            let websys_window = web_sys::window().unwrap();
            let window = window.clone();
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                let size = get_full_size();
                let _ = window.request_inner_size(size);
            }) as Box<dyn FnMut(_)>);
            websys_window
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

        Self { event_loop, window }
    }
}

/// Wrapper type which manages the surface and surface configuration.
///
/// As surface usage varies per platform, wrapping this up cleans up the event loop code.
struct SurfaceWrapper {
    surface: Option<wgpu::Surface<'static>>,
    config: Option<wgpu::SurfaceConfiguration>,
}

impl SurfaceWrapper {
    /// Create a new surface wrapper with no surface or configuration.
    fn new() -> Self {
        Self {
            surface: None,
            config: None,
        }
    }

    /// Called after the instance is created, but before we request an adapter.
    ///
    /// On wasm, we need to create the surface here, as the WebGL backend needs
    /// a surface (and hence a canvas) to be present to create the adapter.
    ///
    /// We cannot unconditionally create a surface here, as Android requires
    /// us to wait until we recieve the `Resumed` event to do so.
    fn pre_adapter(&mut self, instance: &wgpu::Instance, window: Arc<Window>) {
        if cfg!(target_arch = "wasm32") {
            self.surface = Some(instance.create_surface(window).unwrap());
        }
    }

    /// Check if the event is the start condition for the surface.
    fn start_condition(event: &Event<()>) -> bool {
        event == &Event::NewEvents(StartCause::Init)
    }

    /// Called when an event which matches [`Self::start_condition`] is recieved.
    ///
    /// On all native platforms, this is where we create the surface.
    ///
    /// Additionally, we configure the surface based on the (now valid) window size.
    fn resume(&mut self, context: &WgpuContext, window: Arc<Window>, srgb: bool) {
        // Window size is only actually valid after we enter the event loop.
        let window_size = window.inner_size();
        let width = window_size.width.max(1);
        let height = window_size.height.max(1);

        log::info!("Surface resume {window_size:?}");

        // We didn't create the surface in pre_adapter, so we need to do so now.
        if !cfg!(target_arch = "wasm32") {
            self.surface = Some(context.instance.create_surface(window).unwrap());
        }

        // From here on, self.surface should be Some.

        let surface = self.surface.as_ref().unwrap();

        // Get the default configuration,
        let mut config = surface
            .get_default_config(&context.adapter, width, height)
            .expect("Surface isn't supported by the adapter.");
        if srgb {
            // Not all platforms (WebGPU) support sRGB swapchains, so we need to use view formats
            let view_format = config.format.add_srgb_suffix();
            config.view_formats.push(view_format);
        } else {
            // All platforms support non-sRGB swapchains, so we can just use the format directly.
            let format = config.format.remove_srgb_suffix();
            config.format = format;
            config.view_formats.push(format);
        };

        // Comment to disable freerun and enable v-sync. Note that this is only valid in native.
        // #[cfg(not(target_arch = "wasm32"))]
        // {
        //     config.present_mode = wgpu::PresentMode::Immediate;
        // }

        surface.configure(&context.device, &config);
        self.config = Some(config);
    }

    /// Resize the surface, making sure to not resize to zero.
    fn resize(&mut self, context: &WgpuContext, size: winit::dpi::PhysicalSize<u32>) {
        log::info!("Surface resize {size:?}");

        let config = self.config.as_mut().unwrap();
        config.width = size.width.max(1);
        config.height = size.height.max(1);
        let surface = self.surface.as_ref().unwrap();
        surface.configure(&context.device, config);
    }

    /// On suspend on android, we drop the surface, as it's no longer valid.
    ///
    /// A suspend event is always followed by at least one resume event.
    fn suspend(&mut self) {
        self.surface = None;
    }

    fn get(&self) -> Option<&wgpu::Surface> {
        self.surface.as_ref()
    }
}

/// Context containing global wgpu resources.
struct WgpuContext {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
impl WgpuContext {
    /// Initializes the example context.
    async fn init_async(surface: &mut SurfaceWrapper, window: Arc<Window>) -> Self {
        log::info!("Initializing wgpu...");

        let backends: wgpu::Backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
        let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
        let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: wgpu::InstanceFlags::from_build_config().with_env(),
            dx12_shader_compiler,
            gles_minor_version,
        });
        log::info!("Created instance: {:?}", instance);

        surface.pre_adapter(&instance, window);
        // create high performance adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: surface.get(),
                force_fallback_adapter: false,
            })
            .await
            .expect("Unable to find a suitable GPU adapter!");

        log::info!("Adapter: {:?}", adapter.get_info());

        let adapter_info = adapter.get_info();
        log::info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let optional_features = library_bridge::program_optional_features();
        let required_features = library_bridge::program_required_features();
        let adapter_features = adapter.features();
        assert!(
            adapter_features.contains(required_features),
            "Adapter does not support required features for this example: {:?}",
            required_features - adapter_features
        );

        let required_downlevel_capabilities =
            library_bridge::program_required_downlevel_capabilities();
        let downlevel_capabilities = adapter.get_downlevel_capabilities();
        assert!(
            downlevel_capabilities.shader_model >= required_downlevel_capabilities.shader_model,
            "Adapter does not support the minimum shader model required to run this example: {:?}",
            required_downlevel_capabilities.shader_model
        );
        assert!(
            downlevel_capabilities
                .flags
                .contains(required_downlevel_capabilities.flags),
            "Adapter does not support the downlevel capabilities required to run this example: {:?}",
            required_downlevel_capabilities.flags - downlevel_capabilities.flags
        );

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the surface.
        let needed_limits =
            library_bridge::program_required_limits().using_resolution(adapter.limits());

        let trace_dir = std::env::var("WGPU_TRACE");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device Descriptor"),
                    required_features: (optional_features & adapter_features) | required_features,
                    required_limits: needed_limits,
                },
                trace_dir.ok().as_ref().map(std::path::Path::new),
            )
            .await
            .expect("Unable to find a suitable GPU adapter!");

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}

/// Initialize wgpu and run the app.
async fn run(
    // event_loop: EventLoop<()>,
    // window: Rc<Window>,
    data: Arc<Mutex<library_bridge::ReloadFlags>>,
) {
    let window_loop = EventLoopWrapper::new(&library_bridge::get_program_name());
    let mut surface = SurfaceWrapper::new();
    let context = WgpuContext::init_async(&mut surface, window_loop.window.clone()).await;

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::EventLoopExtWebSys;
            let event_loop_function = EventLoop::spawn;
        } else {
            let event_loop_function = EventLoop::run;
        }
    }

    let mut input = WinitInputHelper::new();
    let mut program = None;

    // Create egui state.
    let mut egui_state = egui_winit::State::new(
        egui::Context::default(),
        egui::ViewportId::default(),
        &window_loop.event_loop,
        None,
        None,
    );

    let mut egui_renderer: Option<Renderer> = None;

    #[allow(clippy::let_unit_value)]
    let _ = (event_loop_function)(
        window_loop.event_loop,
        move |event: Event<()>, target: &winit::event_loop::EventLoopWindowTarget<()>| {
            // Poll all events to ensure a maximum framerate.
            // Firefox struggles *a lot* with poll, dropping to less than 10 fps.
            // As such we only enable it in native, since it's not required.
            // Chrome handles poll properly.
            if !cfg!(target_arch = "wasm32") {
                target.set_control_flow(ControlFlow::Poll);
            }

            let mut redraw_requested = false;

            if let Event::WindowEvent {
                event: ref window_event,
                ..
            } = &event
            {
                // ignore event response.
                let _ = egui_state.on_window_event(&window_loop.window, window_event);

                if window_event == &winit::event::WindowEvent::CloseRequested {
                    target.exit();
                }

                redraw_requested = window_event == &winit::event::WindowEvent::RedrawRequested;

                if let winit::event::WindowEvent::Resized(new_size) = window_event {
                    // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                    // See: https://github.com/rust-windowing/winit/issues/208
                    // This solves an issue where the app would panic when minimizing on Windows.

                    let Some(program) = &mut program else {
                        return;
                    };

                    if new_size.width > 0 && new_size.height > 0 {
                        surface.resize(&context, *new_size);
                        library_bridge::resize_program(
                            program,
                            surface.config.as_ref().unwrap(),
                            &context.device,
                            &context.queue,
                        );
                    }
                }
            }

            if SurfaceWrapper::start_condition(&event) {
                surface.resume(&context, window_loop.window.clone(), true);

                if program.is_none() {
                    program = Some(
                        library_bridge::create_program(
                            surface.surface.as_ref().unwrap(),
                            &context.device,
                            &context.adapter,
                            surface.config.as_ref().unwrap(),
                        )
                        .unwrap(),
                    );

                    if let Some(camera) =
                        library_bridge::get_program_camera(program.as_mut().unwrap())
                    {
                        let Some(config) = surface.config.as_mut() else {
                            return;
                        };
                        camera.update(&input, [config.width as f32, config.height as f32]);
                    };
                }

                if egui_renderer.is_none() {
                    egui_renderer = Some(Renderer::new(
                        &context.device,
                        surface.config.as_ref().unwrap().format,
                        None,
                        1,
                    ));
                }
            }

            if event == Event::Suspended {
                surface.suspend();
            }

            if input.update(&event) {
                if input.close_requested() {
                    target.exit();
                }

                if let Some(program) = &mut program {
                    library_bridge::process_input(program, &input);

                    if let Some(camera) = library_bridge::get_program_camera(program) {
                        let Some(config) = surface.config.as_mut() else {
                            return;
                        };
                        camera.update(&input, [config.width as f32, config.height as f32]);
                    };
                };
            }

            if redraw_requested {
                let Some(program) = &mut program else {
                    return;
                };
                let Some(config) = surface.config.as_mut() else {
                    return;
                };
                let Some(surface) = surface.surface.as_ref() else {
                    return;
                };
                let Some(egui_renderer) = egui_renderer.as_mut() else {
                    return;
                };

                // Get the next frame and view.
                let texture = surface.get_current_texture();
                let frame = match texture {
                    Ok(f) => f,
                    Err(e) => {
                        log::warn!("surface lost: window is probably minimized: {e}");
                        return;
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // window_loop.window.request_redraw();

                let mut data = data.lock().unwrap();
                // Reload shaders if needed
                if !data.shaders.is_empty() {
                    log::info!("rebuild shaders {:?}", data.shaders);
                    if let Err(program_error) = library_bridge::update_program_passes(
                        program,
                        surface,
                        &context.device,
                        &context.adapter,
                    ) {
                        log::error!("{program_error:?}");
                    }
                    data.shaders.clear();
                }
                if data.lib == lib::reload_flags::LibState::Reloaded {
                    log::info!("reload lib");
                    if let Err(program_error) = library_bridge::update_program_passes(
                        program,
                        surface,
                        &context.device,
                        &context.adapter,
                    ) {
                        log::error!("{program_error}");
                    }
                    data.lib = library_bridge::LibState::Stable;
                }
                if data.lib == library_bridge::LibState::Stable {
                    // Update the program before drawing.
                    library_bridge::update_program(program, &context.queue);

                    // Render the program first so the ui is on top.
                    library_bridge::render_frame(program, &view, &context.device, &context.queue);

                    // Update the ui before drawing.
                    let input = egui_state.take_egui_input(&window_loop.window);

                    let egui_context = egui_state.egui_ctx();

                    egui_context.begin_frame(input);
                    egui::Window::new(library_bridge::get_program_name()).show(
                        egui_context,
                        |ui| {
                            library_bridge::render_ui(program, ui);
                        },
                    );

                    let output = egui_context.end_frame();
                    let paint_jobs =
                        egui_context.tessellate(output.shapes, egui_context.pixels_per_point());
                    let screen_descriptor = ScreenDescriptor {
                        size_in_pixels: [config.width, config.height],
                        pixels_per_point: egui_context.pixels_per_point(),
                    };

                    // Create a command encoder.
                    let mut encoder = context
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    // Update the egui renderer.
                    {
                        for (id, image_delta) in &output.textures_delta.set {
                            egui_renderer.update_texture(
                                &context.device,
                                &context.queue,
                                *id,
                                image_delta,
                            );
                        }
                        for id in &output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }

                        {
                            egui_renderer.update_buffers(
                                &context.device,
                                &context.queue,
                                &mut encoder,
                                &paint_jobs,
                                &screen_descriptor,
                            );
                        }
                    }

                    // Render ui.
                    {
                        let mut render_pass =
                            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("egui render pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            });

                        egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                    }

                    // Present the frame.
                    context.queue.submit(Some(encoder.finish()));
                    frame.present();
                }
                window_loop.window.request_redraw();
            }
        },
    );
}

/// Create the window depending on the platform.
pub fn start_app(data: Arc<Mutex<lib::reload_flags::ReloadFlags>>) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init().expect("could not initialize logger");
            wasm_bindgen_futures::spawn_local(async move { run(data).await })
        } else {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
            pollster::block_on(run(data));
        }
    }
}
