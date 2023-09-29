#import "demo_raymarching/common.wgsl"


fn sdf_circle(pos: vec2<f32>, origin: vec2<f32>, radius: f32) -> f32 {
    return length(pos - origin) - radius;
}
fn sdf_square(pos: vec2<f32>, origin: vec2<f32>, size: f32, rounding: f32) -> f32 {
    let d = abs(pos - origin) - vec2<f32>(size - rounding, size - rounding);
    return length(max(d, vec2<f32>(0.0, 0.0))) + min(max(d.x, d.y), 0.0) - rounding;
}

fn smiley(pos: vec2<f32>) -> vec2<f32> {
    // glasses
    let g1: f32 = sdf_square(pos, vec2<f32>(-0.3, -0.3), 0.25, 0.04);
    let g2: f32 = sdf_square(pos, vec2<f32>(0.3, -0.3), 0.2, 0.05);
    let g = min(g1, g2);

    // eyes
    let e1: f32 = sdf_circle(pos, vec2<f32>(-0.15, -0.15), 0.05);
    let e2: f32 = sdf_circle(pos, vec2<f32>(0.2, -0.28), 0.05);
    var e = min(e1, e2);
    e = max(g, -e);

    // face
    let f1: f32 = sdf_circle(pos, vec2<f32>(0.0, 0.0), 0.5);
    let f2: f32 = sdf_circle(pos, vec2<f32>(0.0, 0.0), 0.4);
    let f = max(f1, -f2);

    // smile
    let s1: f32 = sdf_circle(pos, vec2<f32>(0.0, 0.05), 0.25);
    let s2: f32 = sdf_circle(pos, vec2<f32>(0.0, -0.2), 0.35);
    let s = max(s1, -s2);

    // nose
    let n1: f32 = sdf_circle(pos, vec2<f32>(0.01, 0.0), 0.04);
    let n2: f32 = sdf_square(pos, vec2<f32>(0.01, 0.03), 0.04, 0.01);
    let n3: f32 = sdf_circle(pos, vec2<f32>(-0.01, 0.06), 0.015);
    let n4: f32 = sdf_circle(pos, vec2<f32>(0.025, 0.06), 0.015);
    let n = max(min(n1, n2), -min(n3, n4));

    var d = min(min(e, f), min(s, n));
    return vec2(d, f1);
}

fn sdf_2d(in: vec2<f32>) -> vec4<f32> {
    let size = vec2<f32>(uniforms.width, uniforms.height);

    // pos is in [0, 1]
    var pos = vec2<f32>(in.x, in.y);
    pos /= size;

    // space repetition
    let pos_x = pos.x;
    let repetition = vec2(5.0);
    pos *= repetition;
    pos = pos % 1.0;

    // rescale to [-1, 1] for sdf
    pos = pos * 2.0 - 1.0;

    // add some animation
    let id_x = floor((pos_x + 1.0) * repetition.x);
    pos.y += sin(uniforms.elapsed) * 0.15 * ((pos.x * 2.0) % 2.0) + cos(id_x * uniforms.elapsed * 0.1) * 0.3;

    // sdf.
    let s = smiley(pos);
    var d = s.x; // distance
    let f = s.y; // circle around the face

    var color = vec3<f32>(1.0, 0.3, 0.7);
    if d < 0.0 {
        // black if inside
        color = vec3<f32>(0.0, 0.0, 0.0);
    }
    var ratio = 0.0;
    if f < 0.0 && d > 0.0 {
        // more lines if inside the face
        ratio = 30.0;
    }
    d = abs(d);
    if fract(ratio * (d - uniforms.elapsed / 30.0)) < 0.01 * ratio {
        // isolines.
        color = vec3<f32>(0.5, 0.0, 0.5);
    }

    return vec4<f32>(color, 1.0);
}

 