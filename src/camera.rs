use crate::SIZE;

use bevy::{
    core::Zeroable,
    prelude::*,
    render::{extract_resource::ExtractResource, render_resource::ShaderType},
};
use bytemuck::Pod;

#[derive(
    ShaderType, Pod, Zeroable, Clone, Copy, Resource, Reflect, ExtractResource, Default, Debug,
)]
#[repr(C)]
pub struct Camera {
    camera_center: [f32; 3],
    _padding1: u32, // https://stackoverflow.com/a/75525055
    viewport_u: [f32; 3],
    _padding2: u32,
    viewport_v: [f32; 3],
    _padding3: u32,
    pixel_delta_u: [f32; 3],
    _padding4: u32,
    pixel_delta_v: [f32; 3],
    _padding5: u32,
    viewport_upper_left: [f32; 3],
    _padding6: u32,
    pixel00_loc: [f32; 3],
    _padding7: u32,
}

impl Camera {
    pub fn create_camera() -> Self {
        let aspect_ratio = SIZE.0 as f32 / SIZE.1 as f32;

        // Camera
        let viewport_height = 2.;
        let viewport_width = viewport_height * aspect_ratio;
        let camera_center = Vec3::splat(0.);
        let focal_length = 1.0;

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = Vec3 {
            x: viewport_width,
            y: 0.,
            z: 0.,
        };
        let viewport_v = Vec3 {
            x: 0.,
            y: -viewport_height,
            z: 0.,
        };

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        let pixel_delta_u = viewport_u / SIZE.0 as f32;
        let pixel_delta_v = viewport_v / SIZE.1 as f32;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = camera_center
            - Vec3 {
                x: 0.,
                y: 0.,
                z: focal_length,
            }
            - viewport_u / 2.
            - viewport_v / 2.;

        let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Camera {
            camera_center: camera_center.into(),
            _padding1: 0,
            viewport_u: viewport_u.into(),
            _padding2: 0,
            viewport_v: viewport_v.into(),
            _padding3: 0,
            pixel_delta_u: pixel_delta_u.into(),
            _padding4: 0,
            pixel_delta_v: pixel_delta_v.into(),
            _padding5: 0,
            viewport_upper_left: viewport_upper_left.into(),
            _padding6: 0,
            pixel00_loc: pixel00_loc.into(),
            _padding7: 0,
        }
    }

    pub fn algined_size() -> u64 {
        std::mem::size_of::<Camera>() as u64 + 4 // todo: figure out alignment, and why this is needed
    }
}
