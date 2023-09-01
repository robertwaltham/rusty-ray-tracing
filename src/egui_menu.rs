use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Button, FontId, RichText},
    EguiContexts, EguiPlugin,
};

use crate::AppState;

pub struct Menu;
impl Plugin for Menu {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, ui_example_system);
        // .init_resource::<UiState>();
    }
}

// #[derive(Default, Resource)]
// struct UiState {
//     label: String,
//     value: f32,
//     inverted: bool,
//     is_window_open: bool,
// }

fn ui_example_system(
    mut contexts: EguiContexts,
    // mut ui_state: ResMut<UiState>,
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    let ctx = contexts.ctx_mut();

    egui::SidePanel::left("side_panel")
        .resizable(false)
        .show(ctx, |ui| {
            ui.heading("Controls");

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
                    "powered by egui",
                    "https://github.com/emilk/egui/",
                ));
            });
        });
}
