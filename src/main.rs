use bevy::{prelude::*, render::render_resource::*};
use rand::prelude::*;
use render::{ComputeShaderPlugin, NoiseImage, RenderImage};

pub mod bevy_menu;
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
        .add_plugins((DefaultPlugins, ComputeShaderPlugin))
        .add_systems(Startup, setup);
    add_gui(&mut app);
    app.run();
}

/*
Note: using egui is currently incompatible with wasm, due to an issue with texture bindings
See: https://github.com/mvlabat/bevy_egui/issues/192, https://github.com/bevyengine/bevy/discussions/9163
*/

#[cfg(target_arch = "wasm32")]
#[allow(unused_variables, unused_mut)]
fn add_gui(app: &mut App) {
    use bevy_menu::Menu;
    app.add_plugins(Menu);
}

#[cfg(not(target_arch = "wasm32"))]
fn add_gui(app: &mut App) {
    use bevy::input::common_conditions::input_toggle_active;
    use bevy_inspector_egui::quick::WorldInspectorPlugin;
    use egui_menu::Menu;
    app.add_plugins((
        Menu,
        WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
    ));
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

    let mut noise_image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );

    for current_pixel in noise_image.data.chunks_exact_mut(4) {
        let random_pixel: [u8; 4] = [random(), random(), random(), 255];
        current_pixel.copy_from_slice(&random_pixel);
    }

    noise_image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let image_handle: Handle<Image> = images.add(noise_image);

    commands.insert_resource(NoiseImage {
        image: image_handle.clone(),
    });
}
