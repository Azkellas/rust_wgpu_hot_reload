use std::sync::{Arc, Mutex};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use crate::hot_lib::library_bridge;

async fn run(
    event_loop: EventLoop<()>,
    window: Window,
    data: Arc<Mutex<lib::helpers::ReloadFlags>>,
) {
    // Create the instance and surface.
    let instance = wgpu::Instance::default();
    let surface = unsafe { instance.create_surface(&window) }.unwrap();

    // Select an adapter and a surface configuration.
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, Some(&surface))
        .await
        .expect("No suitable GPU adapters found on the system!");

    // Create the logical device and command queue
    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Failed to create device");

    // Configure surface.
    let size = window.inner_size();
    let mut config = surface
        .get_default_config(&adapter, size.width, size.height)
        .expect("Surface isn't supported by the adapter.");

    let surface_view_format = config.format.add_srgb_suffix();
    config.view_formats.push(surface_view_format);

    surface.configure(&device, &config);

    // Create our program.
    let mut program = library_bridge::create_program(&surface, &device, &adapter)
        .expect("Failed to create program");

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &program);

        // Poll all events to ensure a maximum framerate.
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                library_bridge::resize_program(&mut program, &config, &device, &queue);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::RedrawEventsCleared => {
                window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                let mut data = data.lock().unwrap();

                // Reload shaders if needed
                if !data.shaders.is_empty() {
                    println!("rebuild shaders {:?}", data.shaders);
                    if let Err(program_error) = library_bridge::update_program_render_pipeline(
                        &mut program,
                        &surface,
                        &device,
                        &adapter,
                    ) {
                        println!("{:?}", program_error);
                    }
                    data.shaders.clear();
                }

                // Rebuild render pipeline if needed
                if data.lib == lib::helpers::LibState::Reloaded {
                    println!("reload lib");
                    if let Err(program_error) = library_bridge::update_program_render_pipeline(
                        &mut program,
                        &surface,
                        &device,
                        &adapter,
                    ) {
                        println!("{:?}", program_error);
                    }
                    data.lib = library_bridge::LibState::Stable;
                }

                // Render a frame if the lib is stable.
                if data.lib == library_bridge::LibState::Stable {
                    let frame = surface
                        .get_current_texture()
                        .expect("Failed to acquire next swap chain texture");
                    let view = frame
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    library_bridge::update_program(&mut program);
                    // program.render(&view, &device, &queue);
                    library_bridge::render_frame(&program, &view, &device, &queue);
                    frame.present();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

pub fn start_app(data: Arc<Mutex<library_bridge::ReloadFlags>>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

        let event_loop = EventLoop::new();
        let mut builder = winit::window::WindowBuilder::new();
        builder = builder.with_title("Demo hot reload");
        let window = builder.build(&event_loop).unwrap();

        pollster::block_on(run(event_loop, window, data));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let event_loop = EventLoop::new();
        let window = winit::window::Window::new(&event_loop).unwrap();

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window, data));
    }
}
