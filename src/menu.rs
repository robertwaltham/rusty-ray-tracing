use crate::AppState;
use bevy::prelude::*;

#[derive(Component)]
pub struct ButtonComponent {
    button_type: ButtonType,
    pressed: bool,
}

enum ButtonType {
    StartButton,
}

pub struct Menu;
impl Plugin for Menu {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, button_interaction)
            .add_systems(Startup, setup_menu);
    }
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
                width: Val::Px(200.),
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
        (
            &Interaction,
            &mut BackgroundColor,
            &mut ButtonComponent,
            &Children,
        ),
        (Changed<Interaction>, With<ButtonComponent>),
    >,
    mut text_query: Query<&mut Text>,
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    for (interaction, mut color, mut button, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                button.pressed = true;
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();

                if button.pressed {
                    match button.button_type {
                        ButtonType::StartButton => match state.get() {
                            AppState::Running => {
                                text.sections[0].value = "Running".to_string();
                                next_state.set(AppState::Waiting);
                            }
                            AppState::Waiting => {
                                next_state.set(AppState::Running);
                                text.sections[0].value = "Waiting".to_string();
                            }
                        },
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
