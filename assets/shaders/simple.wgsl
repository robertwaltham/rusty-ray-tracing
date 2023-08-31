

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

struct Params {
    count: i32,
    size: i32,
    x: i32,
    y: i32,
}

@group(0) @binding(1)
var<uniform> params: Params;


struct Camera {
    focal_length: f32,
    viewport_width: f32,
    viewport_height: f32,
    camera_center: vec3<f32>,
    viewport_u: vec3<f32>,
    viewport_v: vec3<f32>,
    pixel_delta_u: vec3<f32>,
    pixel_delta_v: vec3<f32>,
    viewport_upper_left: vec3<f32>,
    pixel00_loc: vec3<f32>,
    padding_g: f32,
}

@group(0) @binding(2)
var<uniform> camera: Camera;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

fn at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + ray.direction * t;
}

fn ray_colour(ray: Ray) -> vec4<f32> {
    let direction = normalize(ray.direction);
    let value = 0.5 * (direction.y + 1.);
    let rgb = (1.0 - value) * vec3<f32>(1., 1., 1.) + value * vec3<f32>(0.5, 0.7, 1.);
    // let rgb = vec3<f32>(value, 0., 0.);

    return vec4<f32>(rgb, 0.1);
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
    let ray = Ray(camera.camera_center, ray_direction);
    let color = ray_colour(ray);

    storageBarrier();

    textureStore(texture, location, color);
}