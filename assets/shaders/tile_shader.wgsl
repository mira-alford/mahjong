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

    let alpha = smoothstep(0.0, 0.01, -rounded_corners_sdf(uv, size, radius));

    let color = textureSample(material_color_texture, material_color_sampler, mesh.uv)
        * tint;

    return vec4<f32>(color.rgb, alpha);
    // return vec4<f32>(uv, 0.0, alpha);
    // return vec4<f32>(alpha, 0.0, 0.0, 1.0);
}

// adapted from https://www.shadertoy.com/view/ltS3zW
fn rounded_corners_sdf(uv: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    let d = abs(uv) - (size / 2) + vec2<f32>(radius);
    return  min(max(d.x, d.y), 0.0) + length(max(d, vec2<f32>(0.0))) - radius;
}
