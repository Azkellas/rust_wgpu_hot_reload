//! This file should contain the entry points of the library.
//! All reloadable functions should be set up here.
//! To avoid name clashes, the functions in this file
//! should not share names with other functions in the library.

pub mod demo;
mod frame_rate;
pub mod helpers;
mod pass;
pub mod program;

use crate::program::{Program, ProgramError};

/// Specify which program we want to run here.
/// This should also be specified in `src/hot_lib.rs`
pub use crate::demo::DemoProgram as CurrentProgram;

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
) -> Result<CurrentProgram, ProgramError> {
    CurrentProgram::init(surface, device, adapter)
}

/// Contrary to `Program::get_name`, this function returns a String
/// and not a &'static str since we cannot return a static reference
/// from a dynamic library.
#[no_mangle]
pub fn get_program_name(program: &CurrentProgram) -> String {
    program.get_name().into()
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
