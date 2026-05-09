#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var texture_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var front_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var back_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var overlay_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> tint: vec4<f32>;

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {

    let base_color = textureSample(front_texture, texture_sampler, mesh.uv);
    let overlay_color = textureSample(overlay_texture, texture_sampler, mesh.uv);

    var color = vec4<f32>(0.0);

    if base_color.a <= 1e-4 {
        color = vec4<f32>(0.0);
    } else {
        if overlay_color.a <= 1e-4 {
            color = base_color;
        } else {
            color = overlay_color;
        }
    }

    // return vec4<f32>(color, base_color.a);
    return vec4<f32>(color);
    // return vec4<f32>(mesh.uv, 0.0, 1.0);
}
