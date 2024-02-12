//! This file should contain the entry points of the library.
//! All reloadable functions should be set up here.
//! To avoid name clashes, the functions in this file
//! should not share names with other functions in the library.

mod demo_boids;
mod demo_polygon;
mod demo_raymarching;

pub mod camera_control;
mod frame_rate;
pub mod mouse_input;
pub mod program;
pub mod reload_flags;
mod shader_builder;

use crate::program::{Program, ProgramError};

// Any type from libthat is used in the functions signatures in lib.rs should be re-exported here
// and re-imported in hot_lib.rs.
pub use crate::camera_control::CameraLookAt;

/// Specify which program we want to run here.
pub use crate::demo_polygon::DemoPolygonProgram as CurrentProgram;
// pub use crate::demo_boids::DemoBoidsProgram as CurrentProgram;
// pub use crate::demo_raymarching::DemoRaymarchingProgram as CurrentProgram;

/// Hot-reloading does not support generics, so we need to specialize
/// the functions we want to call from the outside.
///
/// # Errors
/// - `ProgramError::ShaderParseError` when the shader could not be compiled.
#[no_mangle]
pub fn create_program(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
    surface_configuration: &wgpu::SurfaceConfiguration,
) -> Result<CurrentProgram, ProgramError> {
    CurrentProgram::init(surface, device, adapter, surface_configuration)
}

/// Contrary to `Program::get_name`, this function returns a String
/// and not a &'static str since we cannot return a static reference
/// from a dynamic library.
#[no_mangle]
pub fn get_program_name() -> String {
    CurrentProgram::get_name().to_owned()
}

/// Resize program. This is called when the main window was resized,
/// to allow programs to update their textures and other data
/// depending on the window size.
#[no_mangle]
pub fn resize_program(
    program: &mut CurrentProgram,
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
/// - `ProgramError::ShaderParseError` when the shader could not be compiled.
#[no_mangle]
pub fn update_program_passes(
    program: &mut CurrentProgram,
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
) -> Result<(), ProgramError> {
    program.update_passes(surface, device, adapter)
}

/// Update program. Called each frame before rendering.
#[no_mangle]
pub fn update_program(program: &mut CurrentProgram, queue: &wgpu::Queue) {
    program.update(queue);
}

/// Render frame.
#[no_mangle]
pub fn render_frame(
    program: &CurrentProgram,
    view: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    program.render(view, device, queue);
}

/// Render ui. Called after `render_frame` to ensure ui is on top.
#[no_mangle]
pub fn render_ui(program: &mut CurrentProgram, ui: &mut egui::Ui) {
    program.draw_ui(ui);
}

#[no_mangle]
pub fn process_input(program: &mut CurrentProgram, input: &winit_input_helper::WinitInputHelper) {
    program.process_input(input);
}

#[no_mangle]
pub fn program_optional_features() -> wgpu::Features {
    CurrentProgram::optional_features()
}

#[no_mangle]
pub fn program_required_features() -> wgpu::Features {
    CurrentProgram::required_features()
}

#[no_mangle]
pub fn program_required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
    CurrentProgram::required_downlevel_capabilities()
}

#[no_mangle]
pub fn program_required_limits() -> wgpu::Limits {
    CurrentProgram::required_limits()
}

#[no_mangle]
pub fn get_program_camera(program: &mut CurrentProgram) -> Option<&mut CameraLookAt> {
    program.get_camera()
}
