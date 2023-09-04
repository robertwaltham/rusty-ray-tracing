

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

struct Params {
    count: i32,
    size: i32,
    x: i32,
    y: i32,
    sphere_count: i32,
    seed: i32,
    samples: i32,
}

@group(0) @binding(1)
var<uniform> params: Params;


struct Camera {
    camera_center: vec3<f32>,
    viewport_u: vec3<f32>,
    viewport_v: vec3<f32>,
    pixel_delta_u: vec3<f32>,
    pixel_delta_v: vec3<f32>,
    viewport_upper_left: vec3<f32>,
    pixel00_loc: vec3<f32>,
}

@group(0) @binding(2)
var<uniform> camera: Camera;


struct Sphere {
    center: vec3<f32>,
    radius: f32,
}

@group(0) @binding(3) 
var<uniform> spheres: array<Sphere, 10>;

// shamelessly copied from https://www.shadertoy.com/view/4ssXRX
fn nrand(n: vec2<f32>) -> f32 {
    return fract(sin(dot(n.xy, vec2(12.9898, 78.233))) * 43758.5453);
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

fn at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + (ray.direction * t);
}

struct HitRecord {
    point: vec3<f32>,
    normal: vec3<f32>,
    t: f32,
    front_face: bool,
    hit: bool
}

fn contains(interval: vec2<f32>, value: f32) -> bool {
    return interval.x <= value && value <= interval.y;
}

fn surrounds(interval: vec2<f32>, value: f32) -> bool {
    return interval.x < value && value < interval.y;
}

fn hit_sphere(sphere: Sphere, ray: Ray, interval: vec2<f32>) -> HitRecord {

    let origin_to_center = ray.origin - sphere.center;
    let a = dot(ray.direction, ray.direction);
    let half_b = dot(origin_to_center, ray.direction);
    let c = dot(origin_to_center, origin_to_center) - pow(sphere.radius, 2.);

    let discriminant = pow(half_b, 2.) - (a * c);
    if discriminant < 0. {
        return HitRecord();
    }

    let sqrt_discriminant = sqrt(discriminant);
    var root = (-half_b - sqrt_discriminant) / a;
    if !surrounds(interval, root) {
        root = (-half_b + sqrt_discriminant) / a;
        if !surrounds(interval, root) {
            return HitRecord();
        }
    }

    let point = at(ray, root);
    var normal = (point - sphere.center) / sphere.radius;
    let front_face = dot(ray.direction, normal) < 0.;

    if !front_face {
        normal = normal * -1.;
    }

    return HitRecord(point, normal, root, front_face, true);
}


@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let color = vec4<f32>(0.5, 0.5, 0.5, 1.0);
    textureStore(texture, location, color);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    var location = vec2<i32>(i32(invocation_id.x + u32(params.x)), i32(invocation_id.y + u32(params.y)));

    let pixel_center = camera.pixel00_loc + (f32(location.x) * camera.pixel_delta_u) + (f32(location.y) * camera.pixel_delta_v);
    let ray_direction = pixel_center - camera.camera_center;
    var color = vec4<f32>(0., 0., 0., 0.);
    for (var i: i32 = 0; i < params.samples; i++) {
        let ray = Ray(camera.camera_center, ray_direction + pixel_sample_square(vec2<f32>(location + i)));
        color += ray_color(ray);
    }
    color /= f32(params.samples);

    storageBarrier();

    textureStore(texture, location, color);
}

fn pixel_sample_square(seed: vec2<f32>) -> vec3<f32> {
    let px = -0.5 + nrand(seed);
    let py = -0.5 + nrand(seed - 1.); // todo: refactor to give a better distribution
    return (camera.pixel_delta_u * px) + (camera.pixel_delta_v * py);
}

fn ray_color(ray: Ray) -> vec4<f32> {
    var closest_hit = HitRecord();
    closest_hit.t = 10000.;
    var closest_sphere = Sphere();

    for (var i: i32 = 0; i < params.sphere_count; i++) {
        let sphere = spheres[i];
        let interval = vec2<f32>(0., closest_hit.t);
        let hit = hit_sphere(sphere, ray, interval);

        if hit.hit && hit.t < closest_hit.t {
            closest_hit = hit;
            closest_sphere = sphere;
        }
    }

    var color: vec4<f32>;
    if closest_hit.hit {
        let normal_color = 0.5 * (closest_hit.normal + 1.);
        color = vec4<f32>(normal_color, 1.);
    } else {
        color = background_color(ray);
    }
    return color;
}

fn background_color(ray: Ray) -> vec4<f32> {
    let direction = normalize(ray.direction);
    let value = (direction.y + 1.) / 2.;
    let rgb = ((1.0 - value) * vec3<f32>(1., 1., 1.)) + (value * vec3<f32>(0.5, 0.7, 1.));
    return vec4<f32>(rgb, 1.);
}