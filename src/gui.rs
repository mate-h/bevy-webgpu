use crate::compute::ComputeShaderSettings;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::MouseMotion,
    prelude::*,
};
use bevy_egui::{egui, EguiContexts, EguiSet};
use bevy_panorbit_camera::PanOrbitCamera;

#[derive(Resource, Default)]
pub struct EguiInteractionState {
    pub wants_focus: bool,
    pub is_dragging: bool,
    pub camera_interaction_active: bool,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct PanOrbitCameraSystemSet;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EguiInteractionState>()
            .add_systems(
                PreUpdate,
                check_egui_wants_focus
                    .after(EguiSet::BeginPass)
                    .before(PanOrbitCameraSystemSet),
            )
            .add_systems(Update, (ui_system, handle_camera_block.after(ui_system)));
    }
}

fn check_egui_wants_focus(mut state: ResMut<EguiInteractionState>, mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    let pointer_state = ctx.input(|i| i.pointer.clone());

    // Start dragging when mouse is pressed over egui
    if ctx.is_pointer_over_area() && pointer_state.any_pressed() {
        state.is_dragging = true;
    }

    // Stop dragging when mouse is released
    if !pointer_state.any_down() {
        state.is_dragging = false;
    }

    // Update wants_focus based on dragging state
    state.wants_focus =
        state.is_dragging || (ctx.is_pointer_over_area() && !state.camera_interaction_active);
}

fn ui_system(
    mut contexts: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
    mut camera_query: Query<&mut PanOrbitCamera>,
    mut shader_settings: Query<&mut ComputeShaderSettings>,
) {
    let ctx = contexts.ctx_mut();

    egui::Window::new("")
        .title_bar(false)
        .default_width(200.0)
        .show(ctx, |ui| {
            if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(fps_value) = fps.smoothed() {
                    ui.label(format!("FPS: {:.1}", fps_value));
                }
            }

            ui.horizontal(|ui| {
                if ui.button("Reset Camera").clicked() {
                    if let Ok(mut camera) = camera_query.get_single_mut() {
                        *camera = PanOrbitCamera {
                            focus: Vec3::ZERO,
                            radius: Some(5.0),
                            yaw: Some(0.0),
                            pitch: Some(std::f32::consts::PI * 0.1),
                            ..Default::default()
                        };
                    }
                }
            });

            if let Ok(camera) = camera_query.get_single() {
                ui.separator();
                ui.label(format!(
                    "Focus: ({:.2}, {:.2}, {:.2})",
                    camera.focus.x, camera.focus.y, camera.focus.z
                ));
                ui.label(format!("Radius: {:.2}", camera.radius.unwrap_or(0.0)));
                ui.label(format!(
                    "Yaw: {:.1}°",
                    camera.yaw.unwrap_or(0.0).to_degrees()
                ));
                ui.label(format!(
                    "Pitch: {:.1}°",
                    camera.pitch.unwrap_or(0.0).to_degrees()
                ));
            }

            ui.separator();
            ui.add(
                egui::Slider::new(
                    &mut shader_settings.get_single_mut().unwrap().value,
                    0.0..=1.0,
                )
                .text("Value"),
            );
        });
}

fn handle_camera_block(
    mut state: ResMut<EguiInteractionState>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera_query: Query<&mut PanOrbitCamera>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    // Check if camera interaction is starting
    if mouse_buttons.just_pressed(MouseButton::Left) && !state.wants_focus {
        state.camera_interaction_active = true;
    }

    // Check if camera interaction is ending
    if mouse_buttons.just_released(MouseButton::Left) {
        state.camera_interaction_active = false;
    }

    if state.wants_focus && !state.camera_interaction_active {
        mouse_motion.clear();

        if let Ok(mut camera) = camera_query.get_single_mut() {
            camera.enabled = false;
        }
    } else if let Ok(mut camera) = camera_query.get_single_mut() {
        camera.enabled = true;
    }
}
