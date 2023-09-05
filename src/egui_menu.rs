use std::any::{self};

use crate::{
    camera::Camera,
    collidables::Spheres,
    render::{Params, RenderTime},
    AppState,
};
use bevy::{prelude::*, reflect::TypeInfo};
use bevy_egui::{
    egui::{self, Button, FontId, RichText},
    EguiContexts, EguiPlugin,
};

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
    mut camera: ResMut<Camera>,
    time: Res<RenderTime>,
    mut params: ResMut<Params>,
    mut spheres: ResMut<Spheres>,
    type_registry: Res<AppTypeRegistry>,
) {
    let ctx = contexts.ctx_mut();

    // let ui_enabled = match state.get() {
    //     AppState::Waiting => true,
    //     AppState::Running => false,
    //     AppState::Done => true,
    //     AppState::Reset => true,
    // };

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

            // ui.set_enabled(ui_enabled);

            ui.allocate_space(egui::Vec2::new(1.0, 20.0));

            ui.heading("Rendering Controls");

            ui.horizontal(|ui| {
                ui.label("sample count");
                ui.add(egui::Slider::new(&mut params.samples, 1..=200).show_value(false));
                ui.label(format!("{}", params.samples));
            });

            ui.horizontal(|ui| {
                ui.label("depth");
                ui.add(egui::Slider::new(&mut params.depth, 1..=100).show_value(false));
                ui.label(format!("{}", params.depth));
            });

            // ui.horizontal(|ui| {
            //     ui.label("step size");
            //     ui.add_enabled(
            //         params.x < 0, // todo: refactor this to be more clear of intent
            //         egui::Slider::from_get_set(6.0..=9.0, |v: Option<f64>| {
            //             if let Some(v) = v {
            //                 params.size = v.exp2() as i32;
            //                 params.x = -params.size;
            //             }
            //             (params.size as f64).log2()
            //         })
            //         .integer()
            //         .show_value(false),
            //     );
            //     ui.label(format!("{}", params.size));
            // });

            ui.allocate_space(egui::Vec2::new(1.0, 20.0));

            ui.heading("Spheres");

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("count");
                    ui.add(egui::Slider::new(&mut params.spheres, 1..=10));
                });

                for i in 0..params.spheres / 2 {
                    ui.label(format!("{}", i));

                    let labels = ["x", "y", "z", "r"];
                    let ranges = [-2.0..=2.0, -2.0..=2.0, -2.0..=0., 0.0..=1.0];

                    for j in 0..4 {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Slider::new(
                                    &mut spheres.spheres[(i * 2) as usize][j],
                                    ranges[j].clone(),
                                )
                                .text(labels[j]),
                            );
                        });
                    }

                    let labels = ["r", "g", "b"];
                    for j in 0..3 {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::Slider::new(
                                    &mut spheres.spheres[((2 * i) + 1) as usize][j],
                                    0.0..=1.0,
                                )
                                .text(labels[j]),
                            );
                        });
                    }
                }
            });
        });

    egui::SidePanel::right("right_panel")
        .resizable(false)
        .min_width(PANEL_WIDTH)
        .show(ctx, |ui| {
            ui.allocate_space(egui::Vec2::new(1.0, 20.0));

            // ui.heading("Params");
            // let param_labels = data_for_resource(&type_registry, params.clone());
            // for (name, value) in param_labels.iter() {
            //     ui.horizontal(|ui| {
            //         ui.label(name);
            //         ui.label(value);
            //     });
            // }

            ui.allocate_space(egui::Vec2::new(1.0, 20.0));

            ui.heading("Camera");
            let camera_labels = data_for_resource(&type_registry, camera.clone());
            for (name, value) in camera_labels.iter() {
                ui.horizontal(|ui| {
                    ui.label(name);
                    ui.label(value);
                });
            }

            ui.allocate_space(egui::Vec2::new(1.0, 10.0));
            ui.label("Camera Center");

            let labels = ["x", "y", "z"];
            for j in 0..3 {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Slider::from_get_set(-1.0..=1.0, |v: Option<f64>| {
                            if let Some(v) = v {
                                camera.camera_center[j] = v as f32;
                            }
                            camera.camera_center[j] as f64
                        })
                        .text(labels[j]),
                    );
                });
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.add(egui::Hyperlink::from_label_and_url(
                    "fork me on github",
                    "https://github.com/robertwaltham/rusty-ray-tracing/",
                ));

                ui.allocate_space(egui::Vec2::new(1.0, 20.0));

                let camera_labels = data_for_resource(&type_registry, time.clone());
                for (name, value) in camera_labels.iter() {
                    ui.horizontal(|ui| {
                        ui.label(name);
                        ui.label(value);
                    });
                }

                ui.heading("Time");
            });
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
                if named_field.name().starts_with('_') {
                    continue;
                }
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
