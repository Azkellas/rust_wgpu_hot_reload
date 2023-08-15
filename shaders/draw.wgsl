struct Uniforms {
  elapsed: f32,
  size: f32,
  speed: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let pi = 3.14159265359;
    let speed = uniforms.speed;
    let radius = uniforms.size;
    let angle = speed * uniforms.elapsed + 2.0 * pi * f32(in_vertex_index) / 3.0 ;
    return vec4<f32>(radius * cos(angle), radius * sin(angle), 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
