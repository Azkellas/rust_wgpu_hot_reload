fn get_camera_ray(x: f32, y: f32) -> vec3<f32> {
    let fov = 60.0;
    let xy: vec2<f32> = vec2(x, y) - vec2(uniforms.width, uniforms.height) / 2.0;
    let z: f32 = uniforms.height / tan(radians(fov) / 2.0);
    return normalize(vec3(xy.x, -xy.y, -z));
}

fn get_view_matrix(eye: vec3<f32>, center: vec3<f32>, up: vec3<f32>) -> mat4x4<f32> {
    let f = normalize(center - eye);
    let s = normalize(cross(f, up));
    let u = cross(s, f);
    return mat4x4(
        vec4(s, 0.0),
        vec4(u, 0.0),
        vec4(-f, 0.0),
        vec4(-dot(eye, s), -dot(eye, u), dot(eye, f), 1.0)
    );
}
