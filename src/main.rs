//! Example showing how to execute compute shaders on demand

use std::borrow::Cow;

use bevy::{
    input::common_conditions::input_toggle_active,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_shader_utils::ShaderUtilsPlugin;

pub struct MainMenu;

#[derive(Component)]
pub struct ButtonComponent {
    button_type: ButtonType,
    pressed: bool,
}

enum ButtonType {
    StartButton,
}

#[derive(Resource, Clone, Deref, ExtractResource, Reflect)]
struct RenderImage {
    image: Handle<Image>,
}

const SIZE: (u32, u32) = (512, 512);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const MENU_BG: Color = Color::rgb(0.1, 0.1, 0.1);
const WORKGROUP_SIZE: u32 = 8;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_plugins(ShaderUtilsPlugin)
        .add_plugins(ComputeShaderPlugin)
        .add_systems(Update, button_interaction)
        .add_systems(Startup, (setup, setup_menu))
        .register_type::<RenderImage>()
        .run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2dBundle::default());

    // This is the texture that will be rendered to.
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let image_handle = images.add(image);

    commands
        .spawn(SpriteBundle {
            texture: image_handle.clone(),
            sprite: Sprite {
                color: HOVERED_BUTTON,
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Render Sprite"));

    commands.insert_resource(RenderImage {
        image: image_handle.clone(),
    });
}

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let start_button = spawn_button(&mut commands, "Start", Color::RED, &asset_server);
    commands
        .entity(start_button)
        .insert(ButtonComponent {
            button_type: ButtonType::StartButton,
            pressed: false,
        })
        .insert(Name::new("Start Render"));

    commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                display: Display::Flex,
                width: Val::Percent(20.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.),
                row_gap: Val::Px(10.),
                ..default()
            },
            background_color: MENU_BG.into(),
            ..default()
        })
        .add_child(start_button)
        .insert(Name::new("Main Menu"));
}

fn spawn_button(
    commands: &mut Commands,
    text: &str,
    color: Color,
    asset_server: &Res<AssetServer>,
) -> Entity {
    commands
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(120.),
                height: Val::Px(65.),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                text,
                TextStyle {
                    font_size: 40.0,
                    color: color,
                    font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                },
            ));
        })
        .id()
}

fn button_interaction(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut ButtonComponent),
        (Changed<Interaction>, With<ButtonComponent>),
    >,
) {
    for (interaction, mut color, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                button.pressed = true;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                if button.pressed {
                    match button.button_type {
                        ButtonType::StartButton => {
                            println!("pressed");
                        }
                    }
                }
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                button.pressed = false;
            }
        }
    }
}

pub struct ComputeShaderPlugin;
impl Plugin for ComputeShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<RenderImage>::default())
            .add_plugins(ExtractResourcePlugin::<ExtractedTime>::default());

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(Render, queue_bind_group.in_set(RenderSet::Queue))
            .add_systems(Render, prepare_time.in_set(RenderSet::Prepare))
            .insert_resource(TimeMeta {
                buffer: None,
                bind_group: None,
            });

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

#[derive(Resource)]
pub struct ComputeShaderPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputeShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout = world.resource::<RenderDevice>().create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
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
                            min_binding_size: BufferSize::new(std::mem::size_of::<f32>() as u64),
                        },
                        count: None,
                    },
                ],
            },
        );
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

#[derive(Resource)]
struct RenderImageBindGroup(BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputeShaderPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    game_of_life_image: Res<RenderImage>,
    render_device: Res<RenderDevice>,
    time_meta: ResMut<TimeMeta>,
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
                resource: time_meta.buffer.as_ref().unwrap().as_entire_binding(),
            },
        ],
    });
    commands.insert_resource(RenderImageBindGroup(bind_group));
}

enum ComputeShaderState {
    Loading,
    Init,
    Update,
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
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
            ComputeShaderState::Update => {
                let update_pipeline = pipeline_cache
                    .get_compute_pipeline(pipeline.update_pipeline)
                    .unwrap();
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
            }
        }

        Ok(())
    }
}

#[derive(Resource, Default)]
struct ExtractedTime {
    seconds_since_startup: f32,
}

impl ExtractResource for ExtractedTime {
    type Source = Time;

    fn extract_resource(time: &Self::Source) -> Self {
        ExtractedTime {
            seconds_since_startup: time.elapsed_seconds(),
        }
    }
}

#[derive(Resource)]
struct TimeMeta {
    buffer: Option<Buffer>,
    bind_group: Option<BindGroup>,
}

// write the extracted time into the corresponding uniform buffer
fn prepare_time(
    time: Res<ExtractedTime>,
    mut time_meta: ResMut<TimeMeta>,
    render_queue: Res<RenderQueue>,
    render_device: Res<RenderDevice>,
) {
    if time_meta.buffer.is_none() {
        time_meta.buffer = Some(render_device.create_buffer(&BufferDescriptor {
            label: Some("time uniform buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }));
    }

    render_queue.write_buffer(
        &time_meta.buffer.as_ref().unwrap(),
        0,
        bevy::core::cast_slice(&[time.seconds_since_startup]),
    );
}
