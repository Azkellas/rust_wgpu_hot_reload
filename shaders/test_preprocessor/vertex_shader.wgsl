#import "test_preprocessor/vertex_output.wgsl"
#import "test_preprocessor/common.wgsl"

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let pi = 3.14159265359;

    // To draw a regular polygon with n edges
    // we draw n triangles with the center of the polygon and two adjacent vertices.
    // calling 0 the center and 1234... the vertices of the polygon
    // we draw 012 023 034 ... 041 which is 045 since angle(n) == angle(0) % 2pi
    // starting from 012345... we compute the triangle_id by dividing by blocks of 3
    // 000 111 222 333 ... (in_vertex_index / 3u)
    let triangle_id = in_vertex_index / 3u;
    // then we get the vertices offset by taking the modulo
    // 012 012 012 012 ... (in_vertex_index % 3u)
    let vertex_offset = in_vertex_index % 3u;
    // then we add both values
    // 012 123 234 345 ... (triangle_id + vertex_offset) 
    let vertex_id = triangle_id + vertex_offset;
    // finally we will just have to consider the first element of the triangle as the center later.
    let vertex_angle = 2.0 * pi * f32(vertex_id) / uniforms.edge_count;

    if vertex_offset > 0u {
        // polygon edge.
        let angle = uniforms.elapsed + vertex_angle;
        let radius = uniforms.size;
        out.position = vec4<f32>(radius * cos(angle), radius * sin(angle), 0.0, 1.0);
    } else {
        // no offset: center of the polygon.
        out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // assign a gradient dependant of the vertex id.
    out.color = vec4<f32>((cos(vertex_angle) + 1.0) / 2.0, 0.0, (sin(vertex_angle) + 1.0) / 2.0, 1.0);

    return out;
}
