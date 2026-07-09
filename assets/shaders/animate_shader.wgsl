#import bevy_pbr::mesh_view_bindings::globals;
#import bevy_pbr::forward_io::VertexOutput;

@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var computed_texture: texture_2d<f32>;

@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var texture_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // offset uv by a sine wave
    let speed = 2.0;
    let time = globals.time * speed;
    var uv = in.uv;
    uv.x += sin(uv.y * 10.0 + time) * 0.1;
    uv.y += cos(uv.x * 10.0 + time) * 0.1;
    return textureSample(computed_texture, texture_sampler, uv);
}
