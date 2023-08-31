//! Example showing how to execute compute shaders on demand

use bevy::{prelude::*, render::render_resource::*};
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
// // use bevy_shader_utils::ShaderUtilsPlugin;
use menu::Menu;
use render::{ComputeShaderPlugin, RenderImage};

pub mod menu;
pub mod render;

#[derive(States, Debug, Default, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Waiting,
    Running,
    Done,
}

const SIZE: (u32, u32) = (512, 512);
const WORKGROUP_SIZE: u32 = 32;
const INIT_WORKGROUP_SIZE: u32 = 8;

fn main() {
    App::new()
        .add_state::<AppState>()
        .add_plugins((DefaultPlugins, ComputeShaderPlugin, Menu))
        // .add_plugins(
        //     WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
        // )
        .add_systems(Startup, setup)
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
                color: Color::GRAY,
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Render Sprite"));

    commands.insert_resource(RenderImage {
        image: image_handle.clone(),
    });
}
