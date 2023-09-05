use bevy::render::{extract_resource::ExtractResource, render_resource::Buffer};

use bevy::{prelude::*, render::render_resource::ShaderType};
use bytemuck::{Pod, Zeroable};

use crate::render::RenderTime;

const MAX_SPHERES: usize = 10;

#[derive(Resource, Debug)]
pub struct SphereBuffer {
    pub buffer: Option<Buffer>,
}

#[derive(
    ShaderType, Pod, Zeroable, Clone, Copy, Resource, Reflect, ExtractResource, Default, Debug,
)]
#[repr(C)]
pub struct Spheres {
    pub spheres: [[f32; 4]; MAX_SPHERES],
}

impl Spheres {
    pub fn default_scene() -> Self {
        let mut spheres = Spheres::default();
        spheres.spheres[0] = [-0.5, 0., -1., 0.5];
        spheres.spheres[1] = [0.7, 0.1, 0.1, 1.0];

        spheres.spheres[2] = [0.5, 0., -1., 0.25];
        spheres.spheres[3] = [0.1, 0.7, 0.1, 1.0];

        spheres.spheres[4] = [0.5, 0., -1., 0.25];
        spheres.spheres[5] = [0.1, 0.1, 0.7, 1.0];

        spheres.spheres[6] = [0., -100.5, -1., 100.];
        spheres.spheres[7] = [0.5, 0.5, 0.5, 1.0];

        spheres
    }
}

pub fn update_spheres(spheres: ResMut<Spheres>, time: Res<RenderTime>) {
    let elapsed = time.time;
    let inner = spheres.into_inner();
    inner.spheres[0][0] = elapsed.sin();
    inner.spheres[2][0] = elapsed.cos();
    inner.spheres[4][1] = elapsed.cos();
}
