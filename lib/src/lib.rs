//! This file should contain the entry points of the library.
//! All reloadable functions should be set up here.
//! To avoid name clashes, the functions in this file
//! should not share names with other functions in the library.
pub mod demo;
pub mod helpers;
pub mod program;

use crate::program::{Program, ProgramError};

/// Specify which program we want to run here.
/// This should also be specified in src/hot_lib.rs
use crate::demo::DemoProgram as CurrentProgram;

/// Hot-reloading does not support generics, so we need to specialize
/// the functions we want to call from the outside.
#[no_mangle]
pub fn create_program(
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
) -> Result<CurrentProgram, ProgramError> {
    CurrentProgram::init(surface, device, adapter)
}

/// Contrary to Program::get_name, this function returns a String
/// and not a &'static str since we cannot return a static reference
/// from a dynamic library.
#[no_mangle]
pub fn get_program_name(program: &CurrentProgram) -> String {
    program.get_name().into()
}

#[no_mangle]
pub fn resize_program(
    program: &mut CurrentProgram,
    surface_configuration: &wgpu::SurfaceConfiguration,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    program.resize(surface_configuration, device, queue)
}

#[no_mangle]
pub fn update_program_render_pipeline(
    program: &mut CurrentProgram,
    surface: &wgpu::Surface,
    device: &wgpu::Device,
    adapter: &wgpu::Adapter,
) -> Result<(), ProgramError> {
    program.update_render_pipeline(surface, device, adapter)
}

#[no_mangle]
pub fn update_program(program: &mut CurrentProgram) {
    program.update()
}

#[no_mangle]
pub fn render_frame<'a, 'b>(program: &'a CurrentProgram, render_pass: &mut wgpu::RenderPass<'b>)
where
    'a: 'b,
{
    program.render(render_pass)
}

#[no_mangle]
pub fn render_ui(program: &mut CurrentProgram, ui: &mut egui::Ui) {
    program.draw_ui(ui)
}
