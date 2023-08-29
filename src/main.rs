//! Example showing how to execute compute shaders on demand

use bevy::{
    input::common_conditions::input_toggle_active,
    prelude::*,
    reflect::TypeUuid,
    render::{render_resource::*, texture::*},
};
use bevy_app_compute::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(TypeUuid)]
#[uuid = "2545ae14-a9bc-4f03-9ea4-4eb43d1075a7"]
struct SimpleShader;

impl ComputeShader for SimpleShader {
    fn shader() -> ShaderRef {
        "shaders/simple.wgsl".into()
    }
}

#[derive(Resource)]
struct SimpleComputeWorker;

impl ComputeWorker for SimpleComputeWorker {
    fn build(world: &mut World) -> AppComputeWorker<Self> {
        let worker = AppComputeWorkerBuilder::new(world)
            .add_uniform("uni", &5.)
            .add_staging("values", &[1., 2., 3., 4.])
            .add_pass::<SimpleShader>([4, 1, 1], &["uni", "values"])
            .one_shot()
            .build();

        worker
    }
}

pub struct MainMenu;

#[derive(Resource)]
pub struct MenuData {
    menu: Entity,
}

#[derive(Component)]
pub struct ButtonComponent {
    button_type: ButtonType,
    pressed: bool,
}

enum ButtonType {
    StartButton,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AppComputePlugin)
        .add_plugin(AppComputeWorkerPlugin::<SimpleComputeWorker>::default())
        .add_plugin(WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)))
        .add_system(read_data)
        .add_system(button_interaction)
        .add_startup_system(setup_menu)
        .add_startup_system(setup)
        .run();
}

fn read_data(mut compute_worker: ResMut<AppComputeWorker<SimpleComputeWorker>>) {
    if !compute_worker.ready() {
        return;
    };

    let result: Vec<f32> = compute_worker.read_vec("values");

    compute_worker.write_slice("values", &result);

    println!("got {:?}", result)
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn(Camera2dBundle::default());

    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0_u8, 0, 0, 255],
        TextureFormat::Bgra8UnormSrgb,
    );

    let image_handle = images.add(image);

    commands
        .spawn(SpriteBundle {
            texture: image_handle,
            sprite: Sprite {
                color: HOVERED_BUTTON,
                custom_size: Some(Vec2::new(512., 512.)),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Render Sprite"));
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const MENU_BG: Color = Color::rgb(0.1, 0.1, 0.1);

fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let start_button = spawn_button(&mut commands, "Start", Color::RED, &asset_server);
    commands
        .entity(start_button)
        .insert(ButtonComponent {
            button_type: ButtonType::StartButton,
            pressed: false,
        })
        .insert(Name::new("Start Render"));

    let menu_entity = commands
        .spawn(NodeBundle {
            style: Style {
                // center button
                display: Display::Flex,
                size: Size {
                    width: Val::Percent(20.),
                    height: Val::Percent(100.),
                },
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_content: AlignContent::Center,
                align_items: AlignItems::Center,
                gap: Size {
                    width: Val::Px(10.),
                    height: Val::Px(10.),
                },
                ..default()
            },
            background_color: MENU_BG.into(),
            ..default()
        })
        .add_child(start_button)
        .insert(Name::new("Main Menu"))
        .id();
    commands.insert_resource(MenuData { menu: menu_entity });
}

fn spawn_button(
    commands: &mut Commands,
    // asset_server: &AssetServer,
    text: &str,
    color: Color,
    asset_server: &Res<AssetServer>,
) -> Entity {
    commands
        .spawn(ButtonBundle {
            style: Style {
                size: Size {
                    width: Val::Px(120.),
                    height: Val::Px(65.),
                },
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
    mut compute_worker: ResMut<AppComputeWorker<SimpleComputeWorker>>,
) {
    for (interaction, mut color, mut button) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                button.pressed = true;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                if button.pressed {
                    match button.button_type {
                        ButtonType::StartButton => {
                            println!("pressed");
                            compute_worker.execute();
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
