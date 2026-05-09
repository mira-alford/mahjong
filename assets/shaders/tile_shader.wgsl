#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var material_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var material_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> tint: vec4<f32>;

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {
    // size of the mahjong peice
    var size = vec2<f32>(1, 4.0 / 3.0); // we know the aspect ratio is 3:4, and bottom edge is 1
     
    let centered_uv = mesh.uv - 1.0 / 2.0;
    let uv = centered_uv * size;

    let radius = 0.3;

    let texture_color = textureSample(material_color_texture, material_color_sampler, mesh.uv);
    let color = texture_color.rgb * tint.rgb;

    return vec4<f32>(color, texture_color.a);
    // return vec4<f32>(uv, 0.0, alpha);
    // return vec4<f32>(alpha, 0.0, 0.0, 1.0);
}
