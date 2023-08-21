#import "test_preprocessor/vertex_output.wgsl"

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
