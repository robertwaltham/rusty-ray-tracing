use std::any::{self};

use bevy::{prelude::*, reflect::TypeInfo};
use bevy_egui::{
    egui::{self, Button, FontId, RichText},
    EguiContexts, EguiPlugin,
};

use crate::render::{Camera, Params};
use crate::AppState;

const PANEL_WIDTH: f32 = 200.;
pub struct Menu;
impl Plugin for Menu {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin).add_systems(Update, ui_system);
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
    camera: Res<Camera>,
    params: Res<Params>,
    type_registry: Res<AppTypeRegistry>,
) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("side_panel")
        .resizable(false)
        .min_width(PANEL_WIDTH)
        .show(ctx, |ui| {
            ui.allocate_space(egui::Vec2::new(1.0, 20.0));

            ui.vertical_centered(|ui| {
                let status_text = match state.get() {
                    AppState::Waiting => "Ready",
                    AppState::Running => "Rendering",
                    AppState::Done => "Done!",
                    AppState::Reset => "Ready", // this should only be for one frame
                };

                ui.label(
                    RichText::new(format!("State: {}", status_text))
                        .font(FontId::proportional(20.0)),
                );

                ui.allocate_space(egui::Vec2::new(1.0, 20.0));

                let button_text = match state.get() {
                    AppState::Waiting => "Start",
                    AppState::Running => "Pause",
                    AppState::Done => "Reset",
                    AppState::Reset => "Start",
                };
                let start_button =
                    Button::new(button_text).min_size(bevy_egui::egui::Vec2::new(100., 30.));
                if ui.add(start_button).clicked() {
                    match state.get() {
                        AppState::Running => next_state.set(AppState::Waiting),
                        AppState::Waiting => next_state.set(AppState::Running),
                        AppState::Done => next_state.set(AppState::Reset),
                        AppState::Reset => {}
                    }
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(egui::Hyperlink::from_label_and_url(
                    "fork me on github",
                    "https://github.com/robertwaltham/rusty-ray-tracing/",
                ));
            });
        });

    egui::SidePanel::right("right_panel")
        .resizable(false)
        .min_width(PANEL_WIDTH)
        .show(ctx, |ui| {
            ui.allocate_space(egui::Vec2::new(1.0, 20.0));

            ui.heading("Params");
            let param_labels = data_for_resource(&type_registry, params.clone());
            for (name, value) in param_labels.iter() {
                ui.horizontal(|ui| {
                    ui.label(name);
                    ui.label(value);
                });
            }

            ui.allocate_space(egui::Vec2::new(1.0, 20.0));

            ui.heading("Camera");
            let camera_labels = data_for_resource(&type_registry, camera.clone());
            for (name, value) in camera_labels.iter() {
                ui.horizontal(|ui| {
                    ui.label(name);
                    ui.label(value);
                });
            }
        });
}

fn data_for_resource<T: Resource + Reflect + GetField>(
    registry: &Res<AppTypeRegistry>,
    resource: T,
) -> Vec<(String, String)> {
    let r = registry.read();

    let resource_info = r.get(any::TypeId::of::<T>()).unwrap().type_info();
    let mut result = Vec::new();

    match resource_info {
        TypeInfo::Struct(info) => {
            for named_field in info.iter() {
                match r.get(named_field.type_id()).unwrap().type_info() {
                    TypeInfo::Value(val) => {
                        macro_rules! type_ui {
                            ( $( $x:ty ),* ) => {
                                $(
                                    if val.is::<$x>() {
                                        result.push((named_field.name().to_string(), resource.get_field::<$x>(named_field.name()).unwrap().to_string()));
                                    }
                                )*
                            };
                        }

                        type_ui!(i32, f32);
                    }

                    TypeInfo::Array(val) => {
                        macro_rules! type_ui {
                            ( $( $x:ty ),* ) => {
                                $(
                                    if val.is::<$x>() {
                                        result.push((named_field.name().to_string(), format!("{:?}", resource.get_field::<$x>(named_field.name()).unwrap())));
                                    }
                                )*
                            };
                        }

                        type_ui!([f32; 3]);
                    }
                    _ => println!("unknown type: {:?}", named_field),
                }
            }
        }
        _ => println!("unknown type: {:?}", resource_info),
    }
    result
}
