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
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    tint: LinearRgba,
}

impl TileMaterial {
    pub fn new(texture: Handle<Image>) -> Self {
        Self {
            texture,
            tint: Color::WHITE.into(),
        }
    }

    pub fn set_tint(&mut self, tint: impl Into<LinearRgba>) {
        self.tint = tint.into();
    }
}

impl Material2d for TileMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_PATH.into()
    }
}
