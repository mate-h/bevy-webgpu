use std::borrow::Cow;

use bevy::{
    asset::AssetMetaCheck,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        globals::{GlobalsBuffer, GlobalsUniform},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
};
use binding_types::{texture_storage_2d, uniform_buffer};

const SHADER_ASSET_PATH: &str = "shaders/compute_shader.wgsl";
const SIZE: (u32, u32) = (256, 256);
const WORKGROUP_SIZE: u32 = 8;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba32Float,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image_handle = images.add(image);
    commands.insert_resource(ComputedTexture {
        texture: image_handle,
    });
}

#[derive(Resource, Clone, ExtractResource)]
pub struct ComputedTexture {
    pub texture: Handle<Image>,
}

pub struct ComputeShaderPlugin;

impl Plugin for ComputeShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup)
            .add_plugins(ExtractResourcePlugin::<ComputedTexture>::default());
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            (
                prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
                update_texture.after(prepare_bind_group),
            ),
        );
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ComputeShaderPipeline>();
    }
}

#[derive(Resource)]
struct ComputeShaderPipeline {
    bind_group_layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for ComputeShaderPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let bind_group_layout = render_device.create_bind_group_layout(
            "simple_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<GlobalsUniform>(false),
                    texture_storage_2d(TextureFormat::Rgba32Float, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        let shader = world.load_asset(SHADER_ASSET_PATH);

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("simple_pipeline".into()),
            layout: vec![bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        });

        ComputeShaderPipeline {
            bind_group_layout,
            pipeline,
        }
    }
}

#[derive(Resource)]
struct ComputeBindGroup(BindGroup);

fn prepare_bind_group(
    mut commands: Commands,
    pipeline: Res<ComputeShaderPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    computed_texture: Res<ComputedTexture>,
    render_device: Res<RenderDevice>,
    globals_buffer: Res<GlobalsBuffer>,
) {
    let view = &gpu_images.get(&computed_texture.texture).unwrap();
    let bind_group = render_device.create_bind_group(
        "simple_bind_group",
        &pipeline.bind_group_layout,
        &BindGroupEntries::sequential((&globals_buffer.buffer, &view.texture_view)),
    );
    commands.insert_resource(ComputeBindGroup(bind_group));
}

fn update_texture(
    pipeline: Res<ComputeShaderPipeline>,
    bind_group: Res<ComputeBindGroup>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    pipeline_cache: Res<PipelineCache>,
) {
    let Some(compute_pipeline) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) else {
        // Pipeline is not ready yet, skip this frame
        return;
    };
    let mut pass = render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    {
        let mut compute_pass = pass.begin_compute_pass(&ComputePassDescriptor::default());
        let compute_pipeline = pipeline_cache
            .get_compute_pipeline(pipeline.pipeline)
            .unwrap();
        compute_pass.set_pipeline(compute_pipeline);
        compute_pass.set_bind_group(0, &bind_group.0, &[]);
        compute_pass.dispatch_workgroups(SIZE.0 / WORKGROUP_SIZE, SIZE.1 / WORKGROUP_SIZE, 1);
    }
    render_queue.submit(std::iter::once(pass.finish()));
}
