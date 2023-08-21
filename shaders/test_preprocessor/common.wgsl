struct Uniforms {
  elapsed: f32,
  size: f32,
  edge_count: f32,
};


@group(0) @binding(0)
var<uniform> uniforms: Uniforms;
