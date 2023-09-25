#import "demo_raymarching/common.wgsl"

#import "demo_raymarching/draw_2d.wgsl"
#import "demo_raymarching/draw_3d.wgsl"

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let color = sdf_2d(in.clip_position.xy);

    let xy = in.clip_position.xy / vec2<f32>(uniforms.width, uniforms.height);
    let color = sdf_3d(in.clip_position.xy);
    return color;
}

 