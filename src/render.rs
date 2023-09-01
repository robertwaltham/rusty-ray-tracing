use crate::{AppState, INIT_WORKGROUP_SIZE, SIZE, WORKGROUP_SIZE};

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
use std::borrow::Cow;

#[derive(Resource, Clone, Deref, ExtractResource, Reflect)]
pub struct RenderImage {
    pub image: Handle<Image>,
}

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

#[derive(
    ShaderType, Pod, Zeroable, Clone, Copy, Resource, Reflect, ExtractResource, Default, Debug,
)]
#[repr(C)]
pub struct Params {
    pub count: i32,
    pub size: i32,
    pub x: i32,
    pub y: i32,
}

#[derive(Resource, Debug)]
struct ParamsBuffer {
    buffer: Option<Buffer>,
}

#[derive(
    ShaderType, Pod, Zeroable, Clone, Copy, Resource, Reflect, ExtractResource, Default, Debug,
)]
#[repr(C)]
pub struct Camera {
    focal_length: f32,
    viewport_width: f32,
    viewport_height: f32,
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
        let pixel_delta_v = viewport_u / SIZE.1 as f32;

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
            focal_length: focal_length,
            viewport_width: viewport_width,
            viewport_height: viewport_height,
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

#[derive(Resource, Debug)]
struct CameraBuffer {
    buffer: Option<Buffer>,
}

pub struct ComputeShaderPlugin;
impl Plugin for ComputeShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<RenderImage>::default(),
            ExtractResourcePlugin::<Params>::default(),
            ExtractResourcePlugin::<Camera>::default(),
        ))
        .register_type::<RenderImage>()
        .register_type::<Params>()
        .insert_resource(Params {
            count: 0,
            size: WORKGROUP_SIZE as i32,
            x: -(WORKGROUP_SIZE as i32),
            y: 0,
        })
        .insert_resource(Camera::create_camera())
        .add_systems(Update, update_params.run_if(in_state(AppState::Running)))
        .add_systems(Last, post_reset.run_if(in_state(AppState::Reset)));

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(Render, queue_bind_group.in_set(RenderSet::Queue))
            .add_systems(Render, prepare_params.in_set(RenderSet::Prepare))
            .add_systems(ExtractSchedule, update_render)
            .insert_resource(RenderState {
                state: AppState::Waiting,
            })
            .insert_resource(ParamsBuffer { buffer: None })
            .insert_resource(CameraBuffer { buffer: None });

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("game_of_life", ComputeShaderNode::default());
        render_graph.add_node_edge(
            "game_of_life",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        )
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

fn update_params(mut params: ResMut<Params>, mut next_state: ResMut<NextState<AppState>>) {
    params.x += WORKGROUP_SIZE as i32;

    if params.x >= SIZE.0 as i32 {
        params.y += WORKGROUP_SIZE as i32;
    }
    params.x = params.x % SIZE.0 as i32;

    params.count += 1;
    if params.count > (SIZE.0 * SIZE.1) as i32 / (WORKGROUP_SIZE * WORKGROUP_SIZE) as i32 {
        next_state.set(AppState::Done);
        params.x = -(WORKGROUP_SIZE as i32);
        params.y = 0;
        params.count = 0;
    }
}

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
    game_of_life_image: Res<RenderImage>,
    render_device: Res<RenderDevice>,
    params_buffer: Res<ParamsBuffer>,
    camera_buffer: Res<CameraBuffer>,
) {
    let view = &gpu_images[&game_of_life_image.image];
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view.texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: params_buffer.buffer.as_ref().unwrap().as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: camera_buffer.buffer.as_ref().unwrap().as_entire_binding(),
            },
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
                    pass.dispatch_workgroups(8, 8, 1); // TODO: match to WORKGROUP_SIZE
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

// write the extracted time into the corresponding uniform buffer
fn prepare_params(
    params: Res<Params>,
    camera: Res<Camera>,
    mut params_buffer: ResMut<ParamsBuffer>,
    mut camera_buffer: ResMut<CameraBuffer>,
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
}

fn post_reset(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Waiting);
}
