#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var material_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var material_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> tint: vec4<f32>;

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {
    return textureSample(material_color_texture, material_color_sampler, mesh.uv) * tint;
}
