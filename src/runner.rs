use egui_wgpu::renderer::{Renderer, ScreenDescriptor};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

use crate::hot_lib::library_bridge;

/// Initialize wgpu and run the app.
async fn run(
    event_loop: EventLoop<()>,
    window: Rc<Window>,
    data: Arc<Mutex<lib::reload_flags::ReloadFlags>>,
) {
    // Create the instance and surface.
    let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);
    let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        dx12_shader_compiler,
    });

    // let instance = wgpu::Instance::default();
    let surface = unsafe { instance.create_surface(window.as_ref()) }.unwrap();

    // Select an adapter and a surface configuration.
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
        .await
        .expect("No suitable GPU adapters found on the system!");

    let optional_features = library_bridge::program_optional_features();
    let required_features = library_bridge::program_required_features();
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features for this example: {:?}",
        required_features - adapter_features
    );

    let required_downlevel_capabilities = library_bridge::program_required_downlevel_capabilities();
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
    // Create the logical device and command queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: (optional_features & adapter_features) | required_features,
                limits: needed_limits,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Unable to find a suitable GPU adapter!");

    // Configure surface.
    let size = window.inner_size();
    let mut config = surface
        .get_default_config(&adapter, size.width, size.height)
        .expect("Surface isn't supported by the adapter.");

    // Comment to disable freerun and enable v-sync. Note that this is only valid in native.
    #[cfg(not(target_arch = "wasm32"))]
    {
        config.present_mode = wgpu::PresentMode::Immediate;
    }

    surface.configure(&device, &config);

    // Create our program.
    let mut program = library_bridge::create_program(&surface, &device, &adapter)
        .expect("Failed to create program");
    // Update window title with program name.
    window.set_title(library_bridge::get_program_name(&program).as_str());

    // Create egui state.
    let mut egui_state = egui_winit::State::new(&event_loop);
    let egui_context = egui::Context::default();
    let mut egui_renderer = Renderer::new(&device, config.format, None, 1);

    let mut mouse_state = lib::mouse_input::MouseState::default();

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (
            &instance,
            &adapter,
            &program,
            &egui_renderer,
            &egui_context,
            &egui_state,
        );

        // Poll all events to ensure a maximum framerate.
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: window_event,
                ..
            } => {
                match window_event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(new_size) => {
                        // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                        // See: https://github.com/rust-windowing/winit/issues/208
                        // This solves an issue where the app would panic when minimizing on Windows.
                        if new_size.width > 0 && new_size.height > 0 {
                            config.width = new_size.width;
                            config.height = new_size.height;
                            surface.configure(&device, &config);
                            library_bridge::resize_program(&mut program, &config, &device, &queue);
                        }
                    }
                    WindowEvent::CursorMoved { .. }
                    | WindowEvent::MouseInput { .. }
                    | WindowEvent::MouseWheel { .. } => {
                        mouse_state.on_window_event(&window_event);
                        if let Some(camera) = library_bridge::get_program_camera(&mut program) {
                            camera.update(&mouse_state, [size.width as f32, size.height as f32]);
                        };
                        // ignore event response.
                        let _ = egui_state.on_event(&egui_context, &window_event);
                    }
                    _ => {
                        // ignore event response.
                        let _ = egui_state.on_event(&egui_context, &window_event);
                    }
                }
            }
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                mouse_state.clear_deltas();
                let mut data = data.lock().unwrap();
                // Reload shaders if needed
                if !data.shaders.is_empty() {
                    log::info!("rebuild shaders {:?}", data.shaders);
                    if let Err(program_error) = library_bridge::update_program_passes(
                        &mut program,
                        &surface,
                        &device,
                        &adapter,
                    ) {
                        log::error!("{program_error:?}");
                    }
                    if let Err(program_error) = library_bridge::update_program_passes(
                        &mut program,
                        &surface,
                        &device,
                        &adapter,
                    ) {
                        log::error!("{program_error:?}");
                    }
                    data.shaders.clear();
                }

                // Rebuild render pipeline if needed
                if data.lib == lib::reload_flags::LibState::Reloaded {
                    log::info!("reload lib");
                    if let Err(program_error) = library_bridge::update_program_passes(
                        &mut program,
                        &surface,
                        &device,
                        &adapter,
                    ) {
                        log::error!("{program_error}");
                    }
                    data.lib = library_bridge::LibState::Stable;
                }

                // Render a frame if the lib is stable.
                if data.lib == library_bridge::LibState::Stable {
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

                    // Create a command encoder.
                    let mut encoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    // Update the program before drawing.
                    library_bridge::update_program(&mut program, &queue);

                    // Render the program first so the ui is on top.
                    library_bridge::render_frame(&program, &view, &device, &queue);

                    // Update the ui before drawing.
                    let input = egui_state.take_egui_input(&window);
                    egui_context.begin_frame(input);
                    egui::Window::new(library_bridge::get_program_name(&program)).show(
                        &egui_context,
                        |ui| {
                            library_bridge::render_ui(&mut program, ui);
                        },
                    );
                    let output = egui_context.end_frame();
                    let paint_jobs = egui_context.tessellate(output.shapes);
                    let screen_descriptor = ScreenDescriptor {
                        size_in_pixels: [config.width, config.height],
                        pixels_per_point: 1.0,
                    };

                    // Update the egui renderer.
                    {
                        for (id, image_delta) in &output.textures_delta.set {
                            egui_renderer.update_texture(&device, &queue, *id, image_delta);
                        }
                        for id in &output.textures_delta.free {
                            egui_renderer.free_texture(id);
                        }

                        {
                            egui_renderer.update_buffers(
                                &device,
                                &queue,
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
                                label: None,
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Load,
                                        store: true,
                                    },
                                })],
                                depth_stencil_attachment: None,
                            });

                        egui_renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
                    }

                    // Present the frame.
                    queue.submit(Some(encoder.finish()));
                    frame.present();
                }
            }
            _ => {}
        }
    });
}

/// Create the window depending on the platform.
pub fn start_app(data: Arc<Mutex<library_bridge::ReloadFlags>>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

        let event_loop = EventLoop::new();
        let builder = winit::window::WindowBuilder::new().with_title("Demo hot reload");
        let window = Rc::new(builder.build(&event_loop).unwrap());

        pollster::block_on(run(event_loop, window, data));
    }
    #[cfg(target_arch = "wasm32")]
    {
        // Create event_loop and window.
        let event_loop = EventLoop::new();
        let window = Rc::new(winit::window::Window::new(&event_loop).unwrap());

        // Add canvas to document body.
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        // Initialize logging.
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");

        // on winit 0.28, the canvas is not resized automatically.
        // this is fixed in 0.29, but egui depends on 0.28 for now
        // so we have to wait. 0.29 should be release late august.
        // In the meantime, this is a workaround.
        // See https://github.com/a-b-street/abstreet/pull/388 for more info.
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

        window.set_inner_size(get_full_size());

        // resize of our winit::Window whenever the browser window changes size.
        {
            let websys_window = web_sys::window().unwrap();
            let window = window.clone();
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
                let size = get_full_size();
                window.set_inner_size(size)
            }) as Box<dyn FnMut(_)>);
            websys_window
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

        // start the app.
        wasm_bindgen_futures::spawn_local(run(event_loop, window, data));
    }
}
