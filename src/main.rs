use bevy::{
    asset::AssetMetaCheck,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use lazy_static::lazy_static;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..Default::default()
            }),
            MaterialPlugin::<CustomMaterial>::default(),
            PanOrbitCameraPlugin,
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, check_for_shader_reload)
        .run();
}

lazy_static! {
    static ref SHADER_RELOAD_TRIGGER: Mutex<Option<String>> = Mutex::new(None);
}

#[wasm_bindgen]
pub fn reload_shader(ptr: *const u8, len: usize) {
    let shader_path = unsafe {
        std::str::from_utf8(std::slice::from_raw_parts(ptr, len)).expect("Invalid UTF-8")
    };
    let mut trigger = SHADER_RELOAD_TRIGGER.lock().unwrap();
    *trigger = Some(shader_path.to_string());
}

fn should_reload_shader() -> Option<String> {
    let mut trigger = SHADER_RELOAD_TRIGGER.lock().unwrap();
    trigger.take()
}

fn check_for_shader_reload(world: &mut World) {
    if let Some(shader_path) = should_reload_shader() {
        let asset_server = world.resource::<AssetServer>();
        asset_server.reload(&shader_path);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    run();
}

#[cfg(target_arch = "wasm32")]
fn main() {}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Cuboid::default()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: materials.add(CustomMaterial {}),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/animate_shader.wgsl".into()
    }
}
