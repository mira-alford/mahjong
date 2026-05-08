use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{Material2d, Material2dPlugin},
};

const SHADER_PATH: &str = "shaders/tile_shader.wgsl";

pub struct TileMaterialPlugin;

impl Plugin for TileMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<TileMaterial>::default());
    }
}

#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct TileMaterial {
    #[texture(1)]
    #[sampler(2)]
    texture: Handle<Image>,
}

impl TileMaterial {
    pub fn new(texture: Handle<Image>) -> Self {
        Self { texture }
    }
}

impl Material2d for TileMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_PATH.into()
    }
}
