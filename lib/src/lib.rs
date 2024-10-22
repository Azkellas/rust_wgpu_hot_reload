//! This file should contain the entry points of the library.
//! All reloadable functions should be set up here.
//! To avoid name clashes, the functions in this file
//! should not share names with other functions in the library.

mod current_input;
pub mod winit_input_helper;

pub mod camera_control;
pub mod demo_pipelines;
mod frame_rate;
pub mod mouse_input;
pub mod pipeline;
pub mod reload_flags;
mod shader_builder;

use crate::pipeline::{PipelineError, PipelineFuncs};

/// default shader builder for this library's shaders.
pub type ShaderBuilderForLibrary = ShaderBuilderFor<LibraryShaders>;

// Any type from libthat is used in the functions signatures in lib.rs should be re-exported here
// and re-imported in hot_lib.rs.
pub use crate::camera_control::CameraLookAt;
/// Specify which program we want to run here.
pub use demo_pipelines::polygon::Pipeline as CurrentPipeline;
use shader_builder::{LibraryShaders, ShaderBuilderFor};
// pub use demo_pipelines::boids::Pipeline as CurrentPipeline;
// pub use demo_pipelines::raymarching::Pipeline as CurrentPipeline;

/// Hot-reloading does not support generics, so we need to specialize
/// the functions we want to call from the outside.
///
/// # Errors
/// - `PipelineError::ShaderParseError` when the shader could not be compiled.
#[no_mangle]
pub fn create_program(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    surface_configuration: &wgpu::SurfaceConfiguration,
) -> Result<CurrentPipeline, PipelineError> {
    CurrentPipeline::init(surface, device, adapter, surface_configuration)
}

/// Contrary to `PipelineFuncs::get_name`, this function returns a String
/// and not a &'static str since we cannot return a static reference
/// from a dynamic library.
#[no_mangle]
pub fn get_program_name() -> String {
    CurrentPipeline::get_name().to_owned()
}

/// Resize program. This is called when the main window was resized,
/// to allow pipelines to update their textures and other data
/// depending on the window size.
#[no_mangle]
pub fn resize_program(
    program: &mut CurrentPipeline,
    surface_configuration: &wgpu::SurfaceConfiguration,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    program.resize(surface_configuration, device, queue);
}

/// Update program passes. Called when a shader needs to be reloaded
/// or the libray is done reloading,
///
/// # Errors
/// - `PipelineError::ShaderParseError` when the shader could not be compiled.
#[no_mangle]
pub fn update_program_passes(
    program: &mut CurrentPipeline,
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
) -> Result<(), PipelineError> {
    program.update_passes(surface, device, adapter)
}

/// Update program. Called each frame before rendering.
#[no_mangle]
pub fn update_program(program: &mut CurrentPipeline, queue: &wgpu::Queue) {
    program.update(queue);
}

/// Render frame.
#[no_mangle]
pub fn render_frame(
    program: &CurrentPipeline,
    view: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    program.render(view, device, queue);
}

/// Render ui. Called after `render_frame` to ensure ui is on top.
#[no_mangle]
pub fn render_ui(program: &mut CurrentPipeline, ui: &mut egui::Ui) {
    program.draw_ui(ui);
}

#[no_mangle]
pub fn process_input(program: &mut CurrentPipeline, input: &winit_input_helper::WinitInputHelper) {
    program.process_input(input);
}

#[no_mangle]
pub fn program_optional_features() -> wgpu::Features {
    CurrentPipeline::optional_features()
}

#[no_mangle]
pub fn program_required_features() -> wgpu::Features {
    CurrentPipeline::required_features()
}

#[no_mangle]
pub fn program_required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
    CurrentPipeline::required_downlevel_capabilities()
}

#[no_mangle]
pub fn program_required_limits() -> wgpu::Limits {
    CurrentPipeline::required_limits()
}

#[no_mangle]
pub fn get_program_camera(program: &mut CurrentPipeline) -> Option<&mut CameraLookAt> {
    program.get_camera()
}
