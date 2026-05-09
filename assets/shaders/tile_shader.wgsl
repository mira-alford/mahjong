#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::view,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var texture_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var front_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var back_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var overlay_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<uniform> tint: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> tilt: vec2<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var<uniform> scale: vec2<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var<uniform> flags: u32;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) blend_color: vec4<f32>,
    @location(2) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.uv = vertex.uv;

    let position = vertex.position * vec3<f32>(scale, 1.0);
        
    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(position, 1.0)
    );
    let clip_pos = mesh_functions::mesh2d_position_world_to_clip(out.world_position);
    out.position = clip_pos;

    return out;
}

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {

    var color = vec4<f32>(0.0);

    if flags == 0 {
        let base_color = textureSample(front_texture, texture_sampler, mesh.uv);
        let overlay_color = textureSample(overlay_texture, texture_sampler, mesh.uv);
        color = mix(base_color, overlay_color, overlay_color.a);
    } else {
        color = textureSample(back_texture, texture_sampler, mesh.uv);
    }

    // return vec4<f32>(color, base_color.a);
    return vec4<f32>(color.rgb * tint.rgb, color.a);
    // return vec4<f32>(mesh.uv, 0.0, 1.0);
}
