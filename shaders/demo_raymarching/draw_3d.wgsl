#import "demo_raymarching/camera.wgsl"

const EPSILON = 0.001;

fn sdf_sphere(p: vec3<f32>, origin: vec3<f32>, radius: f32) -> f32 {
    return length(p - origin) - radius;
}
fn sdf_round_box(position: vec3<f32>, origin: vec3<f32>, radius: f32, rounding: f32) -> f32 {
    let q = abs(position - origin) - vec3(radius);
    return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0) - rounding;
}
fn sdf_cube(p: vec3<f32>, origin: vec3<f32>, radius: f32) -> f32 {
    let d = abs(p - origin) - vec3(radius);
    let insideDistance = min(max(d.x, max(d.y, d.z)), 0.0);
    let outsideDistance = length(max(d, vec3(0.0)));
    return insideDistance + outsideDistance;
}

// colored sdf: vec4 == (distance, r, g, b)
fn sdf_col(a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    if a.x < b.x {
        return a;
    }
    return b;
}

fn sdf_scene(position: vec3<f32>) -> vec4<f32> {
    // repetition.
    let s = vec3(7.0);
    let id = round(position / s);
    var pos = position - s * id;

    pos += 0.5 * sin((uniforms.elapsed + (id.x + id.y + id.z)) * 10.0);

    // colored sdf: (distance, r, g, b)
    var d = vec4(1000.0, 0.0, 0.0, 0.0);
 
    // body
    d = sdf_col(d, vec4(sdf_sphere(pos, vec3(0.0, 0.0, 0.0), 1.0), 0.6, 0.0, 0.8));

    // eyes
    let sqrt2 = sqrt(2.0) / 2.0;
    d = sdf_col(d, vec4(sdf_round_box(pos, vec3(-(sqrt2 - 0.2), sqrt2, 0.7), 0.2, 0.15), 0.02, 0.02, 0.02));
    d = sdf_col(d, vec4(sdf_round_box(pos, vec3(sqrt2 - 0.2, sqrt2, 0.7), 0.2, 0.15), 0.02, 0.02, 0.02));

    // nose
    d = sdf_col(d, vec4(sdf_sphere(pos, vec3(0.0, 0.2, 0.7), 0.35), 1.0, 0.3, 0.0));

    // smile
    let big_sphere = sdf_sphere(pos, vec3(0.0, 0.7, 0.4), 1.0);
    let small_sphere = sdf_sphere(pos, vec3(0.0, 0.35, 0.4), 0.8);
    let smile = max(-big_sphere, small_sphere);

    d = sdf_col(d, vec4(smile, 4.0, 0.0, 0.0));

    return d;
}

fn estimate_normal(p: vec3<f32>) -> vec3<f32> {
    return normalize(vec3(
        sdf_scene(vec3(p.x + EPSILON, p.y, p.z)).x - sdf_scene(vec3(p.x - EPSILON, p.y, p.z)).x,
        sdf_scene(vec3(p.x, p.y + EPSILON, p.z)).x - sdf_scene(vec3(p.x, p.y - EPSILON, p.z)).x,
        sdf_scene(vec3(p.x, p.y, p.z + EPSILON)).x - sdf_scene(vec3(p.x, p.y, p.z - EPSILON)).x
    ));
}

fn phong_lighting(k_d: f32, k_s: f32, alpha: f32, position: vec3<f32>, eye: vec3<f32>, light_pos: vec3<f32>, light_intensity: vec3<f32>) -> vec3<f32> {
    let N = estimate_normal(position);
    let L = normalize(light_pos - position);
    let V = normalize(eye - position);
    let R = normalize(reflect(-L, N));

    let dotLN = dot(L, N);
    let dotRV = dot(R, V);

    if dotLN < 0.0 {
        // Light not visible from this point on the surface
        return vec3(0.0, 0.0, 0.0);
    }

    if dotRV < 0.0 {
        // Light reflection in opposite direction as viewer, apply only diffuse
        // component
        return light_intensity * (k_d * dotLN);
    }
    return light_intensity * (k_d * dotLN + k_s * pow(dotRV, alpha));
}


// entry point of the 3d raymarching.
fn sdf_3d(p: vec2<f32>) -> vec4<f32> {
    var time: f32 = uniforms.elapsed;

    // camera look at.
    let look_at: vec3<f32> = uniforms.camera_center.xyz;

    // compute direction.
    let longitude = uniforms.camera_longitude;
    let latitude = uniforms.camera_latitude;
    let angle = vec3(
        cos(longitude) * cos(latitude),
        sin(latitude),
        sin(longitude) * cos(latitude)
    );
    let up = vec3(0.0, 1.0, 0.0);

    // camera position.
    var eye: vec3<f32> = look_at + angle * uniforms.camera_distance;

    // compute ray direction.
    let matrix: mat4x4<f32> = get_view_matrix(eye, look_at, up);
    var ray: vec3<f32> = get_camera_ray(p.x, p.y); // ray in camera space.
    ray = (matrix * vec4(ray, 0.0)).xyz; // ray in world space.

    // actual ray marching.
    let MAX_STEPS = 100;
    var position = eye;
    var dist = vec4(0.0);
    for (var i = 0; i < MAX_STEPS; i++) {
        dist = sdf_scene(position);
        if dist.x < EPSILON {
                break;
        }
        position += ray * dist.x;
    }

    var color = dist.yzw;
    if dist.x < EPSILON {
        // add lighting only if we hit something.
        color = phong_lighting(0.8, 0.5, 50.0, position, eye, vec3(-5.0, 5.0, 5.0), dist.yzw);
    }

    // add fog.
    let view_dist = length(position - eye);
    let fog = exp(-0.04 * view_dist);
    color = mix(color, vec3(0.00, 0.0, 0.05), 1.0 - fog);

    return vec4<f32>(color, 1.0);
}
