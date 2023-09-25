// Vertex shader
struct Uniforms {
  elapsed: f32,
  width: f32,
  height: f32,
  _padding: f32,  // padding to 16 bytes, required for WebGL.
};

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

