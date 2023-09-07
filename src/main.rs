use bevy::{prelude::*, render::render_resource::*};
use egui_menu::Menu;
use render::{ComputeShaderPlugin, RenderImage};

pub mod camera;
pub mod collidables;
pub mod egui_menu;
pub mod render;

#[derive(States, Debug, Default, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    Waiting,
    Running,
    Done,
    Reset,
}

const SIZE: (u32, u32) = (512, 512);
const INIT_WORKGROUP_SIZE: u32 = 8;

fn main() {
    let mut app = App::new();
    app.add_state::<AppState>()
        .add_plugins((DefaultPlugins, ComputeShaderPlugin, Menu))
        .add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2dBundle::default());

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
            sprite: Sprite { ..default() },
            ..default()
        })
        .insert(Name::new("Render Sprite"));

    commands.insert_resource(RenderImage {
        image: image_handle.clone(),
    });
}
