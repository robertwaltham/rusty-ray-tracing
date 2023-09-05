

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
    depth: i32,
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
    color: vec4<f32>,
}

@group(0) @binding(3) 
var<uniform> spheres: array<Sphere, 10>;


@group(0) @binding(4)
var noise_texture: texture_storage_2d<rgba8unorm, read>;


// https://www.shadertoy.com/view/4djSRW
// fn nrand() -> f32 {
//     seed += 1.;
//     var p3 = fract(vec3<f32>(seed.xyx) * .1031);
//     p3 += dot(p3, p3.yzx + 33.33);
//     return fract((p3.x + p3.y) * p3.z);
// }

fn nrand() -> f32 {
    let pixel = textureLoad(noise_texture, seed);

    seed.x = (seed.x + 1) % 512;
    seed.y = (seed.y + 1) % 512;

    return ((pixel.x + pixel.y + pixel.z) / 1.5) - 1.;
}
fn nrand_vec3() -> vec3<f32> {
    return vec3<f32>(nrand(), nrand(), nrand());
}
var<workgroup> seed: vec2<i32>;

fn rand_in_unit_sphere() -> vec3<f32> {

    // bail out after 100 reps
    for (var i = 0; i < 100; i++) {
        let v = nrand_vec3();
        if v.x * v.x + v.y * v.y + v.z * v.z < 1. {
            return v;
        }
    }

    return nrand_vec3();
}

fn random_on_hemisphere(normal: vec3<f32>) -> vec3<f32> {
    let on_unit_sphere = rand_in_unit_sphere();
    if dot(normalize(on_unit_sphere), normal) > 0.0 {
        return on_unit_sphere;
    } else {
        return -on_unit_sphere;
    }
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
    color: vec4<f32>,
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

    // todo: sphere color calc
    // let normal_color = vec4<f32>(0.5 * (normal + 1.), 1.);
    // let normal_color = vec4<f32>(0.5, 0.5, 0.5, 1.);

    return HitRecord(point, normal, sphere.color, root, front_face, true);
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
    seed = location; // set initial random seed
    let pixel_center = camera.pixel00_loc + (f32(location.x) * camera.pixel_delta_u) + (f32(location.y) * camera.pixel_delta_v);
    let ray_direction = pixel_center - camera.camera_center;
    var color = vec4<f32>(nrand(), 0., 0., 1.);
    for (var i: i32 = 0; i < params.samples; i++) {
        let ray = Ray(camera.camera_center, ray_direction + pixel_sample_square());
        color += ray_color(ray);
    }
    color /= f32(params.samples);



    storageBarrier();

    textureStore(texture, location, color);
}

fn pixel_sample_square() -> vec3<f32> {
    let px = -0.5 + nrand();
    let py = -0.5 + nrand();
    return (camera.pixel_delta_u * px) + (camera.pixel_delta_v * py);
}

fn test_hit_spheres(ray: Ray) -> HitRecord {

    var closest_hit = HitRecord();
    closest_hit.t = 10000.;

    for (var i: i32 = 0; i < params.sphere_count / 2; i++) {
        let sphere = spheres[i];
        let interval = vec2<f32>(0.01, closest_hit.t);
        let hit = hit_sphere(sphere, ray, interval);

        if hit.hit && hit.t < closest_hit.t {
            closest_hit = hit;
        }
    }

    return closest_hit;
}

fn ray_color(ray: Ray) -> vec4<f32> {

    var ray = ray;

    var hit_colours = array<vec4<f32>, 10>();
    var hits = 0;

    let bg_color = background_color(ray);
    var has_hit = false;
    while hits < params.depth {
        let closest_hit = test_hit_spheres(ray);

        if closest_hit.hit {
            hit_colours[hits] = closest_hit.color;

            let direction = random_on_hemisphere(closest_hit.normal);
            ray = Ray(closest_hit.point, direction);
            hits += 1;
            has_hit = true;
        } else {

            if hits > 0 {
                hit_colours[hits] = bg_color;
                hits += 1;
            }

            break;
        }
    }

    var color = vec4<f32>(0., 0., 0., 1.);

    if has_hit {
        for (var i: i32 = 0; i < hits; i++) {
            color += hit_colours[i]; /// pow(2., f32(i + 1));
        }
        return color / f32(hits);
    } else {
        return bg_color;
    }
}

fn background_color(ray: Ray) -> vec4<f32> {
    let direction = normalize(ray.direction);
    let value = (direction.y + 1.) / 2.;
    let rgb = ((1.0 - value) * vec3<f32>(1., 1., 1.)) + (value * vec3<f32>(0.5, 0.7, 1.));
    return vec4<f32>(rgb, 1.);
}