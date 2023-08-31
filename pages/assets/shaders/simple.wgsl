// #import bevy_shader_utils::simplex_noise_3d simplex_noise_3d


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


@compute @workgroup_size(8, 8, 1)
fn init(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let color = vec4<f32>(0.5, 0.5, 0.5, 1.0);

    textureStore(texture, location, color);
}

@compute @workgroup_size(8, 8, 1)
fn update(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    var location = vec2<i32>(i32(invocation_id.x + u32(params.x)), i32(invocation_id.y + u32(params.y)));

    // let location_for_noise = vec3<f32>(f32(invocation_id.x + u32(params.x)) * 0.02, f32(invocation_id.y + u32(params.y)) * 0.02, 1.0);
    // let noise = simplex_noise_3d(location_for_noise);

    // let color = vec4<f32>(noise, noise * noise, noise * 0.2, 1.0);

        let color = vec4<f32>(0.0, 0.5, 0.0, 1.0);


    storageBarrier();

    textureStore(texture, location, color);
}