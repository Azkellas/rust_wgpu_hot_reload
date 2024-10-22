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
/// Specify which pipeline we want to run here.
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
pub fn create_pipeline(
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
pub fn get_pipeline_name() -> String {
    CurrentPipeline::get_name().to_owned()
}

/// Resize pipeline. This is called when the main window was resized,
/// to allow pipelines to update their textures and other data
/// depending on the window size.
#[no_mangle]
pub fn resize_pipeline(
    pipeline: &mut CurrentPipeline,
    surface_configuration: &wgpu::SurfaceConfiguration,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    pipeline.resize(surface_configuration, device, queue);
}

/// Update pipeline passes. Called when a shader needs to be reloaded
/// or the libray is done reloading,
///
/// # Errors
/// - `PipelineError::ShaderParseError` when the shader could not be compiled.
#[no_mangle]
pub fn update_pipeline_passes(
    pipeline: &mut CurrentPipeline,
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
) -> Result<(), PipelineError> {
    pipeline.update_passes(surface, device, adapter)
}

/// Update pipeline. Called each frame before rendering.
#[no_mangle]
pub fn update_pipeline(pipeline: &mut CurrentPipeline, queue: &wgpu::Queue) {
    pipeline.update(queue);
}

/// Render frame.
#[no_mangle]
pub fn render_frame(
    pipeline: &CurrentPipeline,
    view: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    pipeline.render(view, device, queue);
}

/// Render ui. Called after `render_frame` to ensure ui is on top.
#[no_mangle]
pub fn render_ui(pipeline: &mut CurrentPipeline, ui: &mut egui::Ui) {
    pipeline.draw_ui(ui);
}

#[no_mangle]
pub fn process_input(pipeline: &mut CurrentPipeline, input: &winit_input_helper::WinitInputHelper) {
    pipeline.process_input(input);
}

#[no_mangle]
pub fn pipeline_optional_features() -> wgpu::Features {
    CurrentPipeline::optional_features()
}

#[no_mangle]
pub fn pipeline_required_features() -> wgpu::Features {
    CurrentPipeline::required_features()
}

#[no_mangle]
pub fn pipeline_required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
    CurrentPipeline::required_downlevel_capabilities()
}

#[no_mangle]
pub fn pipeline_required_limits() -> wgpu::Limits {
    CurrentPipeline::required_limits()
}

#[no_mangle]
pub fn get_pipeline_camera(pipeline: &mut CurrentPipeline) -> Option<&mut CameraLookAt> {
    pipeline.get_camera()
}
