use bevy::{
    asset::AssetMetaCheck,
    camera::Camera3dDepthTextureUsage,
    dev_tools::infinite_grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings},
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, TextureUsages},
    shader::ShaderRef,
};
use bevy_egui::EguiPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use wasm_bindgen::prelude::*;

mod compute;
mod gui;
mod post_process;
mod shader_reload;

use compute::{ComputeShaderPlugin, ComputedTexture};
use gui::GuiAppPlugin;
use post_process::{PostProcessPlugin, PostProcessSettings};
use shader_reload::ShaderReloadPlugin;

#[wasm_bindgen]
pub fn run() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..Default::default()
            }),
            InfiniteGridPlugin,
            MaterialPlugin::<CustomMaterial>::default(),
            PanOrbitCameraPlugin,
            FrameTimeDiagnosticsPlugin::default(),
            ShaderReloadPlugin,
            ComputeShaderPlugin,
            EguiPlugin::default(),
            GuiAppPlugin,
            PostProcessPlugin,
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
    computed_texture: Res<ComputedTexture>,
) {
    commands.spawn((
        InfiniteGrid,
        InfiniteGridSettings {
            fadeout_distance: 100.0,
            ..default()
        },
    ));

    commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(CustomMaterial {
            computed_texture: computed_texture.texture.clone(),
        })),
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        Camera3d {
            depth_texture_usages: Camera3dDepthTextureUsage::from(
                TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            ),
            ..default()
        },
        PanOrbitCamera::default(),
        PostProcessSettings::default(),
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
