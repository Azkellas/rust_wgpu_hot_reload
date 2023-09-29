// Vertex shader
struct Uniforms {
  elapsed: f32,
  width: f32,
  height: f32,
  camera_angle: f32,
  camera_center: vec3<f32>,
  camera_height: f32,
  camera_distance: f32,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

