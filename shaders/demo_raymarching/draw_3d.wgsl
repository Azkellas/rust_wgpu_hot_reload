const EPSILON = 0.001;

fn get_camera_ray(x: f32, y: f32) -> vec3<f32> {
    let fov = 60.0;
    let xy = vec2(x, y) - vec2(uniforms.width, uniforms.height) / 2.0;
    let z = uniforms.height / tan(radians(fov) / 2.0);
    return normalize(vec3(xy, -z));
}


fn estimateNormal(p: vec3<f32>) -> vec3<f32> {
    return normalize(vec3(
        sdf_scene(vec3(p.x + EPSILON, p.y, p.z)).x - sdf_scene(vec3(p.x - EPSILON, p.y, p.z)).x,
        sdf_scene(vec3(p.x, p.y + EPSILON, p.z)).x - sdf_scene(vec3(p.x, p.y - EPSILON, p.z)).x,
        sdf_scene(vec3(p.x, p.y, p.z + EPSILON)).x - sdf_scene(vec3(p.x, p.y, p.z - EPSILON)).x
    ));
}

fn sdf_sphere(p: vec3<f32>, origin: vec3<f32>, radius: f32) -> f32 {
    return length(p - origin) - radius;
}

fn sdf_round_box(position: vec3<f32>, origin: vec3<f32>, radius: f32, rounding: f32) -> f32 {
    let q = abs(position - origin) - vec3(radius);
    return length(max(q, vec3(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0) - rounding;
}


fn sdf_cube(p: vec3<f32>, origin: vec3<f32>, radius: f32) -> f32 {
    // If d.x < 0, then -1 < p.x < 1, and same logic applies to p.y, p.z
    // So if all components of d are negative, then p is inside the unit cube
    let d = abs(p - origin) - vec3(radius);
    
    // Assuming p is inside the cube, how far is it from the surface?
    // Result will be negative or zero.
    let insideDistance = min(max(d.x, max(d.y, d.z)), 0.0);
    
    // Assuming p is outside the cube, how far is it from the surface?
    // Result will be positive or zero.
    let outsideDistance = length(max(d, vec3(0.0)));

    return insideDistance + outsideDistance;
}

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

    pos += sin(uniforms.elapsed * (id.z + id.x + id.y) * 2.0);

    var d = vec4(1000.0, 0.0, 0.0, 0.0);
 
    // body
    d = sdf_col(d, vec4(sdf_sphere(pos, vec3(0.0, 0.0, 0.0), 1.0), 0.6, 0.0, 0.8));

    // eyes
    let sqrt2 = sqrt(2.0) / 2.0;
    d = sdf_col(d, vec4(sdf_round_box(pos, vec3(sqrt2 - 0.2, -sqrt2, 0.7), 0.2, 0.15), 0.02, 0.02, 0.02));
    d = sdf_col(d, vec4(sdf_round_box(pos, vec3(-sqrt2 + 0.2, -sqrt2, 0.7), 0.2, 0.15), 0.02, 0.02, 0.02));

    // nose
    d = sdf_col(d, vec4(sdf_sphere(pos, vec3(0.0, -0.2, 0.7), 0.35), 1.0, 0.3, 0.0));

    // smile
    let big_sphere = sdf_sphere(pos, vec3(0.0, -0.7, 0.6), 1.0);
    let small_sphere = sdf_sphere(pos, vec3(0.0, -0.35, 0.6), 0.8);
    let smile = max(-big_sphere, small_sphere);

    d = sdf_col(d, vec4(smile, 4.0, 0.0, 0.0));

    return d;
}

fn phong_lighting(k_d: f32, k_s: f32, alpha: f32, position: vec3<f32>, eye: vec3<f32>, light_pos: vec3<f32>, light_intensity: vec3<f32>) -> vec3<f32> {
    let N = estimateNormal(position);
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

fn viewMatrix(eye: vec3<f32>, center: vec3<f32>, up: vec3<f32>) -> mat4x4<f32> {
    let f = normalize(center - eye);
    let s = normalize(cross(f, up));
    let u = cross(s, f);
    return mat4x4(
        vec4(s, 0.0),
        vec4(u, 0.0),
        vec4(-f, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );
}


fn sdf_3d(p: vec2<f32>) -> vec4<f32> {
    var ray = get_camera_ray(p.x, p.y);
    var time: f32 = uniforms.elapsed;
    // time = 0.0;
    var eye = vec3<f32>(-2.6 + sin(time), -2.4 + cos(time), 6.0);
    let look_at = vec3(2.0 * sin(time), 2.0 * cos(2.0 * time), 0.0);

    let up = vec3(0.0, 0.2 * cos(time * 0.2), abs(sin(time * 0.7)) - 0.3);
    let matrix = viewMatrix(eye, look_at, normalize(up));
    ray = (matrix * vec4(ray, 0.0)).xyz;

    let MAX_STEPS = 100;
    var position = eye;
    for (var i = 0; i < MAX_STEPS; i++) {
        let dist = sdf_scene(position);
        if dist.x < 0.001 {
                break;
        }
        position += ray * dist.x;
    }

    let dist = sdf_scene(position);

    var color = dist.yzw;
    if dist.x < EPSILON {
        color = phong_lighting(0.8, 0.5, 50.0, position, eye, vec3(-2.0, -3.0, 4.0), dist.yzw);
    }
    let view_dist = length(position - eye);
    let fog = exp(-0.04 * view_dist);
    color = mix(color, vec3(0.00, 0.0, 0.05), 1.0 - fog);

    return vec4<f32>(color, 1.0);
}
