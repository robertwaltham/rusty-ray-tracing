

@group(0) @binding(0)
var texture: texture_storage_2d<rgba8unorm, write>;

struct Params {
    count: i32,
    size: i32,
    x: i32,
    y: i32,
    sphere_count: i32,
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


struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

fn at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + (ray.direction * t);
}

fn ray_colour(ray: Ray) -> vec4<f32> {
    let direction = normalize(ray.direction);
    let value = (direction.y + 1.) / 2.;
    let rgb = ((1.0 - value) * vec3<f32>(1., 1., 1.)) + (value * vec3<f32>(0.5, 0.7, 1.));
    return vec4<f32>(rgb, 1.);
}

struct HitRecord {
    point: vec3<f32>,
    normal: vec3<f32>,
    t: f32,
    front_face: bool,
    hit: bool
}

fn hit_sphere(sphere: Sphere, ray: Ray, ray_tmin: f32, ray_tmax: f32) -> HitRecord {

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
    if root <= ray_tmin || ray_tmax <= root {
        root = (-half_b + sqrt_discriminant) / a;
        if root <= ray_tmin || ray_tmax <= root {
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
    let ray = Ray(camera.camera_center, ray_direction);


    var closest_hit = HitRecord();
    closest_hit.t = 10000.;
    var closest_sphere = Sphere();

    for (var i: i32 = 0; i < params.sphere_count; i++) {
        let sphere = spheres[i];
        let hit = hit_sphere(sphere, ray, 0., closest_hit.t);

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
        color = ray_colour(ray);
    }

    storageBarrier();

    textureStore(texture, location, color);
}