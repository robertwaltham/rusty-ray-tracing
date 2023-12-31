use crate::{camera::Camera, collidables::*, AppState, INIT_WORKGROUP_SIZE, SIZE};

use bevy::{
    core::Zeroable,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
};
use bytemuck::{bytes_of, Pod};
use std::{borrow::Cow, collections::VecDeque};

#[derive(Resource, Clone, Deref, ExtractResource, Reflect)]
pub struct RenderImage {
    pub image: Handle<Image>,
}

// todo: storage buffers for web

// const NOISE_BUFFER_SIZE: u64 = 20;
// #[derive(Resource, Clone, Deref, ExtractResource)]
// pub struct NoiseBuffer {
//     pub buffer: Option<Buffer>,
// }

#[derive(Resource, Default)]
struct RenderState {
    state: AppState,
}

#[derive(Resource)]
struct RenderImageBindGroup(BindGroup);

enum ComputeShaderState {
    Loading,
    Init,
    Update,
}

#[derive(ShaderType, Pod, Zeroable, Clone, Copy, Resource, Reflect, ExtractResource, Debug)]
#[repr(C)]
pub struct Params {
    pub count: i32,
    pub size: i32,
    pub x: i32,
    pub y: i32,
    pub spheres: i32,
    pub seed: i32,
    pub samples: i32,
    pub depth: i32,
    pub render_mode: i32,
    pub _padding1: i32,
    pub _padding2: i32,
    pub _padding3: i32,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            count: 0,
            size: 512,
            x: 0,
            y: 0,
            spheres: 8,
            seed: 0,
            samples: 25,
            depth: 3,
            render_mode: 0,
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
        }
    }
}

#[derive(Resource, Debug)]
struct ParamsBuffer {
    buffer: Option<Buffer>,
}

#[derive(Resource, Debug)]
struct CameraBuffer {
    buffer: Option<Buffer>,
}

#[derive(Resource, Debug, Default, Reflect, Clone)]
pub struct RenderTime {
    pub time: f32,
    pub frames: i32,
    pub min_frame: f32,
    pub max_frame: f32,
    pub avg_frame: f32,
    pub avg_fps: f32,
    pub avg_fps_10: f32,
    pub _last_10: VecDeque<f32>,
}

pub struct ComputeShaderPlugin;
impl Plugin for ComputeShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<RenderImage>::default(),
            ExtractResourcePlugin::<Params>::default(),
            ExtractResourcePlugin::<Camera>::default(),
            ExtractResourcePlugin::<Spheres>::default(),
        ))
        .register_type::<RenderImage>()
        .register_type::<Params>()
        .register_type::<Camera>()
        .register_type::<RenderTime>()
        .register_type::<[f32; 3]>()
        .insert_resource(Params::default())
        .insert_resource(Camera::create_camera())
        .insert_resource(Spheres::default_scene())
        .insert_resource(RenderTime::default())
        .add_systems(
            Update,
            (update_time, update_spheres).run_if(in_state(AppState::Running)),
        )
        .add_systems(
            Last,
            (post_reset, reset_time).run_if(in_state(AppState::Reset)),
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(Render, queue_bind_group.in_set(RenderSet::Queue))
            .add_systems(Render, prepare_params.in_set(RenderSet::Prepare))
            .add_systems(ExtractSchedule, update_render)
            .insert_resource(RenderState {
                state: AppState::Waiting,
            })
            .insert_resource(ParamsBuffer { buffer: None })
            .insert_resource(SphereBuffer { buffer: None })
            .insert_resource(CameraBuffer { buffer: None });
        // todo: storage buffers for web
        // .insert_resource(NoiseBuffer { buffer: None });

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("ray_trace_node", ComputeShaderNode::default());
        render_graph.add_node_edge(
            "ray_trace_node",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ComputeShaderPipeline>();
    }
}

fn update_render(mut commands: Commands, state: Extract<Res<State<AppState>>>) {
    commands.insert_resource(RenderState {
        state: state.get().clone(),
    });
}

fn update_time(time: Res<Time>, mut render_time: ResMut<RenderTime>) {
    let delta = time.delta_seconds();
    render_time.time += delta;
    render_time.frames += 1;
    render_time.min_frame = std::cmp::min_by(render_time.min_frame, delta, |x, y| {
        x.partial_cmp(y).unwrap()
    });
    render_time.max_frame = std::cmp::max_by(render_time.min_frame, delta, |x, y| {
        x.partial_cmp(y).unwrap()
    });
    render_time.avg_frame = render_time.time / render_time.frames as f32;
    render_time.avg_fps = 1. / render_time.avg_frame;
    render_time._last_10.push_front(delta);
    if render_time._last_10.len() > 11 {
        render_time._last_10.pop_back();
    }

    render_time.avg_fps_10 = 1.
        / (render_time._last_10.iter().fold(0., |acc, v| acc + v)
            / render_time._last_10.len() as f32);
}

fn reset_time(mut render_time: ResMut<RenderTime>) {
    render_time.time = 0.;
    render_time.frames = 0;
    render_time.min_frame = 0.;
    render_time.max_frame = 0.;
    render_time.avg_frame = 0.;
    render_time.avg_fps = 0.;
    render_time.avg_fps_10 = 0.;
    render_time._last_10.clear();
}

// todo: separate modes between one shot and continuous

// fn update_params(mut params: ResMut<Params>, mut next_state: ResMut<NextState<AppState>>) {
// params.x += params.size;

// if params.x >= SIZE.0 as i32 {
//     params.y += params.size;
// }
// params.x = params.x % SIZE.0 as i32;

// params.count += 1;
// if params.count > (SIZE.0 * SIZE.1) as i32 / (params.size * params.size) {
//     // next_state.set(AppState::Done);
//     params.x = -(params.size as i32);
//     params.y = 0;
//     params.count = 0;
//     params.seed += 1;
// }
// }

#[derive(Resource)]
pub struct ComputeShaderPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputeShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: StorageTextureAccess::WriteOnly,
                                format: TextureFormat::Rgba8Unorm,
                                view_dimension: TextureViewDimension::D2,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: BufferSize::new(
                                    std::mem::size_of::<Params>() as u64
                                ),
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: BufferSize::new(Camera::algined_size()),
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 3,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: BufferSize::new(
                                    std::mem::size_of::<Spheres>() as u64
                                ),
                            },
                            count: None,
                        },
                        // todo: storage textures for web
                        // BindGroupLayoutEntry {
                        //     binding: 4,
                        //     visibility: ShaderStages::COMPUTE,
                        //     ty: BindingType::Buffer {
                        //         ty: BufferBindingType::Storage { read_only: true },
                        //         has_dynamic_offset: false,
                        //         min_binding_size: BufferSize::new(NOISE_BUFFER_SIZE * 4),
                        //     },
                        //     count: None,
                        // },
                    ],
                });
        let shader = world.resource::<AssetServer>().load("shaders/simple.wgsl");
        let pipeline_cache = world.resource_mut::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
            push_constant_ranges: vec![],
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
            push_constant_ranges: vec![],
        });

        ComputeShaderPipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputeShaderPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    output_image: Res<RenderImage>,
    render_device: Res<RenderDevice>,
    params_buffer: Res<ParamsBuffer>,
    camera_buffer: Res<CameraBuffer>,
    spheres_buffer: Res<SphereBuffer>,
    // noise_buffer: Res<NoiseBuffer>,
) {
    let output_view = &gpu_images[&output_image.image];

    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&output_view.texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: params_buffer.buffer.as_ref().unwrap().as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: camera_buffer.buffer.as_ref().unwrap().as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: spheres_buffer.buffer.as_ref().unwrap().as_entire_binding(),
            },
            // todo: storage textures for web

            // BindGroupEntry {
            //     binding: 4,
            //     resource: noise_buffer.buffer.as_ref().unwrap().as_entire_binding(),
            // },
        ],
    });
    commands.insert_resource(RenderImageBindGroup(bind_group));
}

struct ComputeShaderNode {
    state: ComputeShaderState,
}

impl Default for ComputeShaderNode {
    fn default() -> Self {
        Self {
            state: ComputeShaderState::Loading,
        }
    }
}

impl render_graph::Node for ComputeShaderNode {
    fn update(&mut self, world: &mut World) {
        let pipeline = world.resource::<ComputeShaderPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        // if the corresponding pipeline has loaded, transition to the next stage
        match self.state {
            ComputeShaderState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.init_pipeline)
                {
                    self.state = ComputeShaderState::Init;
                }
            }
            ComputeShaderState::Init => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
                {
                    self.state = ComputeShaderState::Update;
                }
            }
            ComputeShaderState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let texture_bind_group = &world.resource::<RenderImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ComputeShaderPipeline>();
        let state = &world.resource::<RenderState>().state;
        let window_size = &world.resource::<Params>().size;
        let workgroup_size = (window_size / 8) as u32;
        let mut pass = render_context
            .command_encoder()
            .begin_compute_pass(&ComputePassDescriptor::default());

        pass.set_bind_group(0, texture_bind_group, &[]);

        // select the pipeline based on the current state
        match self.state {
            ComputeShaderState::Loading => {}
            ComputeShaderState::Init => {
                let init_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.init_pipeline)
                    .unwrap();
                pass.set_pipeline(init_pipeline);
                pass.dispatch_workgroups(
                    SIZE.0 / INIT_WORKGROUP_SIZE,
                    SIZE.1 / INIT_WORKGROUP_SIZE,
                    1,
                );
            }
            ComputeShaderState::Update => {
                if state == &AppState::Running {
                    let update_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.update_pipeline)
                        .unwrap();
                    pass.set_pipeline(update_pipeline);
                    pass.dispatch_workgroups(workgroup_size, workgroup_size, 1);
                } else if state == &AppState::Reset {
                    let init_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.init_pipeline)
                        .unwrap();
                    pass.set_pipeline(init_pipeline);
                    pass.dispatch_workgroups(
                        SIZE.0 / INIT_WORKGROUP_SIZE,
                        SIZE.1 / INIT_WORKGROUP_SIZE,
                        1,
                    );
                }
            }
        }

        Ok(())
    }
}

fn prepare_params(
    params: Res<Params>,
    camera: Res<Camera>,
    spheres: Res<Spheres>,
    mut params_buffer: ResMut<ParamsBuffer>,
    mut camera_buffer: ResMut<CameraBuffer>,
    mut spheres_buffer: ResMut<SphereBuffer>,
    // mut noise_buffer: ResMut<NoiseBuffer>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    if params_buffer.buffer.is_none() {
        params_buffer.buffer = Some(render_device.create_buffer(&BufferDescriptor {
            label: Some("params buffer"),
            size: std::mem::size_of::<Params>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }

    if camera_buffer.buffer.is_none() {
        camera_buffer.buffer = Some(render_device.create_buffer(&BufferDescriptor {
            label: Some("camera buffer"),
            size: Camera::algined_size(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }

    if spheres_buffer.buffer.is_none() {
        spheres_buffer.buffer = Some(render_device.create_buffer(&BufferDescriptor {
            label: Some("spheres buffer"),
            size: std::mem::size_of::<Spheres>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }

    // todo: storage buffers for web

    // if noise_buffer.buffer.is_none() {
    //     noise_buffer.buffer = Some(render_device.create_buffer(&BufferDescriptor {
    //         label: Some("noise buffer"),
    //         size: NOISE_BUFFER_SIZE * 4,
    //         usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
    //         mapped_at_creation: false,
    //     }));
    // }

    // let mut random_number: Vec<f32> = Vec::new();
    // for _ in 0..NOISE_BUFFER_SIZE {
    //     random_number.push(random());
    // }
    // render_queue.write_buffer(
    //     &noise_buffer.buffer.as_ref().unwrap(),
    //     0,
    //     cast_slice(random_number.as_slice()),
    // );

    render_queue.write_buffer(
        &params_buffer.buffer.as_ref().unwrap(),
        0,
        bytes_of(params.as_ref()),
    );

    render_queue.write_buffer(
        &camera_buffer.buffer.as_ref().unwrap(),
        0,
        bytes_of(camera.as_ref()),
    );

    render_queue.write_buffer(
        &spheres_buffer.buffer.as_ref().unwrap(),
        0,
        bytes_of(spheres.as_ref()),
    );
}

fn post_reset(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Waiting);
}
