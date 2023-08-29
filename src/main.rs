//! Example showing how to execute compute shaders on demand

use bevy::{
    input::common_conditions::input_toggle_active,
    prelude::*,
    render::{extract_resource::ExtractResource, render_resource::*},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::Escape)),
        )
        .add_systems(Update, button_interaction)
        .add_systems(Startup, (setup, setup_menu))
        .register_type::<RenderImage>()
        .run();
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
            texture: image_handle.clone(),
            sprite: Sprite {
                color: HOVERED_BUTTON,
                custom_size: Some(Vec2::new(512., 512.)),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Render Sprite"));

    commands.insert_resource(RenderImage {
        image: image_handle.clone(),
    });
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
