use std::borrow::Cow;

use bevy::{
    asset::RenderAssetUsages,
    core_pipeline::schedule::camera_driver,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::{
            ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
            UniformComponentPlugin,
        },
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        globals::{GlobalsBuffer, GlobalsUniform},
        render_asset::RenderAssets,
        render_resource::{
            binding_types::{texture_storage_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderGraph},
        texture::GpuImage,
    },
    shader::ShaderCacheError,
};

const SHADER_ASSET_PATH: &str = "shaders/compute_shader.wgsl";
const SIZE: (u32, u32) = (256, 256);
const WORKGROUP_SIZE: u32 = 8;

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub struct ComputeShaderSettings {
    pub value: f32,
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_target_texture(SIZE.0, SIZE.1, TextureFormat::Rgba32Float, None);
    image.asset_usage = RenderAssetUsages::RENDER_WORLD;
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image_handle = images.add(image);
    commands.insert_resource(ComputedTexture {
        texture: image_handle,
    });
    commands.spawn(ComputeShaderSettings { value: 1.0 });
}

#[derive(Resource, Clone, ExtractResource)]
pub struct ComputedTexture {
    pub texture: Handle<Image>,
}

pub struct ComputeShaderPlugin;

impl Plugin for ComputeShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup).add_plugins((
            ExtractResourcePlugin::<ComputedTexture>::default(),
            ExtractComponentPlugin::<ComputeShaderSettings>::default(),
            UniformComponentPlugin::<ComputeShaderSettings>::default(),
        ));

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ComputeShaderState>()
            .add_systems(RenderStartup, init_compute_pipeline)
            .add_systems(
                Render,
                (
                    update_compute_state.in_set(RenderSystems::Prepare),
                    prepare_bind_group.in_set(RenderSystems::PrepareBindGroups),
                ),
            )
            .add_systems(RenderGraph, compute_system.before(camera_driver));
    }
}

#[derive(Resource)]
struct ComputeShaderPipeline {
    bind_group_layout: BindGroupLayoutDescriptor,
    pipeline: CachedComputePipelineId,
}

#[derive(Resource, Default)]
enum ComputeShaderState {
    #[default]
    Loading,
    Ready,
}

fn init_compute_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pipeline_cache: Res<PipelineCache>,
) {
    let bind_group_layout = BindGroupLayoutDescriptor::new(
        "compute_shader_bind_group_layout",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                uniform_buffer::<GlobalsUniform>(false),
                texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                uniform_buffer::<ComputeShaderSettings>(true),
            ),
        ),
    );

    let shader = asset_server.load(SHADER_ASSET_PATH);

    let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some("compute_shader_pipeline".into()),
        layout: vec![bind_group_layout.clone()],
        shader,
        entry_point: Some(Cow::from("main")),
        ..default()
    });

    commands.insert_resource(ComputeShaderPipeline {
        bind_group_layout,
        pipeline,
    });
}

#[derive(Resource)]
struct ComputeBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputeShaderPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    computed_texture: Res<ComputedTexture>,
    globals_buffer: Res<GlobalsBuffer>,
    settings_uniforms: Res<ComponentUniforms<ComputeShaderSettings>>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
) {
    let Some(view) = gpu_images.get(&computed_texture.texture) else {
        return;
    };
    let Some(settings_binding) = settings_uniforms.binding() else {
        return;
    };

    let bind_group = render_device.create_bind_group(
        "compute_shader_bind_group",
        &pipeline_cache.get_bind_group_layout(&pipeline.bind_group_layout),
        &BindGroupEntries::sequential((
            &globals_buffer.buffer,
            &view.texture_view,
            settings_binding.clone(),
        )),
    );

    commands.insert_resource(ComputeBindGroup(bind_group));
}

fn update_compute_state(
    pipeline: Res<ComputeShaderPipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut state: ResMut<ComputeShaderState>,
) {
    if matches!(*state, ComputeShaderState::Ready) {
        return;
    }

    match pipeline_cache.get_compute_pipeline_state(pipeline.pipeline) {
        CachedPipelineState::Ok(_) => {
            *state = ComputeShaderState::Ready;
        }
        CachedPipelineState::Err(ShaderCacheError::ShaderNotLoaded(_)) => {}
        CachedPipelineState::Err(err) => {
            panic!("Initializing assets/{SHADER_ASSET_PATH}:\n{err}");
        }
        _ => {}
    }
}

fn compute_system(
    mut render_context: RenderContext,
    pipeline: Res<ComputeShaderPipeline>,
    pipeline_cache: Res<PipelineCache>,
    state: Res<ComputeShaderState>,
    bind_group: Option<Res<ComputeBindGroup>>,
    settings_index: Query<&DynamicUniformIndex<ComputeShaderSettings>>,
) {
    if !matches!(*state, ComputeShaderState::Ready) {
        return;
    }
    let Some(bind_group) = bind_group else {
        return;
    };
    let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) else {
        return;
    };
    let Ok(settings_index) = settings_index.single() else {
        return;
    };

    let mut pass = render_context
        .command_encoder()
        .begin_compute_pass(&ComputePassDescriptor::default());

    pass.set_pipeline(compute_pipeline);
    pass.set_bind_group(0, &bind_group.0, &[settings_index.index()]);
    pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
}
