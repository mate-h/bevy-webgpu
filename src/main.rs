use bevy::{
    asset::AssetMetaCheck,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use wasm_bindgen::prelude::*;
mod shader_reload;
use shader_reload::ShaderReloadPlugin;
mod compute;
use compute::{ComputeShaderPlugin, ComputedTexture};

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
            ShaderReloadPlugin,
            ComputeShaderPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
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
    mut computed_texture: ResMut<ComputedTexture>,
) {
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Cuboid::default()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        material: materials.add(CustomMaterial {
            computed_texture: computed_texture.texture.clone(),
        }),
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
struct CustomMaterial {
    #[texture(1)]
    #[sampler(2)]
    computed_texture: Handle<Image>,
}

impl Material for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/animate_shader.wgsl".into()
    }
}
