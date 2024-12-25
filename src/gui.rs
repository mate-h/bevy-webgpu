use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy_panorbit_camera::PanOrbitCamera;
use bevy::render::extract_resource::ExtractResource;

pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ui_system)
            .insert_resource(ShaderValue(0.5)); // Default value
    }
}

#[derive(Resource, Clone, ExtractResource)]
pub struct ShaderValue(pub f32);

fn ui_system(
    mut contexts: EguiContexts, 
    diagnostics: Res<DiagnosticsStore>,
    mut camera_query: Query<&mut PanOrbitCamera>,
    mut shader_value: ResMut<ShaderValue>,
) {
    egui::Window::new("")
    .title_bar(false)
    .default_width(200.0)
    .show(contexts.ctx_mut(), |ui| {
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
            ui.label(format!("Focus: ({:.2}, {:.2}, {:.2})", camera.focus.x, camera.focus.y, camera.focus.z));
            ui.label(format!("Radius: {:.2}", camera.radius.unwrap_or(0.0)));
            ui.label(format!("Yaw: {:.1}°", camera.yaw.unwrap_or(0.0).to_degrees()));
            ui.label(format!("Pitch: {:.1}°", camera.pitch.unwrap_or(0.0).to_degrees()));
        }
    });
} 