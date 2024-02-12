// Vertex shader
struct Uniforms {
  camera_center: vec4<f32>,
  camera_longitude: f32,
  camera_latitude: f32,
  camera_distance: f32,
  width: f32,
  height: f32,
  elapsed: f32,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

