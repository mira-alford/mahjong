use bevy::{
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

const SHADER_PATH: &str = "shaders/tile_shader.wgsl";

pub struct TileMaterialPlugin;

impl Plugin for TileMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<TileMaterial>::default());
    }
}

/// [`Material2d`] handling rendering, animations and effects on [`Tile`]s. Use
/// the methods on [`TileMaterial`] to control it
#[derive(Asset, AsBindGroup, Debug, Clone, TypePath)]
pub struct TileMaterial {
    #[sampler(0)]
    #[texture(1)]
    front_texture: Handle<Image>,
    #[texture(2)]
    back_texture: Handle<Image>,
    #[texture(3)]
    overlay_texture: Option<Handle<Image>>,
    #[uniform(4)]
    tint: LinearRgba,
    #[uniform(5)]
    tilt: Vec2,
    #[uniform(6)]
    scale: Vec2,
}

impl TileMaterial {
    pub fn new(
        front_texture: Handle<Image>,
        back_texture: Handle<Image>,
        overlay_texture: Option<Handle<Image>>,
    ) -> Self {
        Self {
            tint: Color::WHITE.into(),
            front_texture,
            back_texture,
            overlay_texture,
            tilt: Vec2::ZERO,
            scale: Vec2::ONE,
        }
    }

    // tint setters and getters
    pub fn set_tint(&mut self, tint: impl Into<LinearRgba>) {
        self.tint = tint.into();
    }

    pub fn get_tint(self) -> LinearRgba {
        self.tint
    }

    #[must_use]
    pub fn get_tint_mut(&mut self) -> &mut LinearRgba {
        &mut self.tint
    }

    // scale getters and setters
    pub fn set_scale(&mut self, scale: &Vec2) {
        self.scale = *scale
    }

    pub fn get_scale(self) -> Vec2 {
        self.scale
    }

    #[must_use]
    pub fn get_scale_mut(&mut self) -> &mut Vec2 {
        &mut self.scale
    }

    // tilt getters and setters
    pub fn set_tilt(&mut self, tilt: &Vec2) {
        self.tilt = *tilt
    }

    pub fn get_tilt(self) -> Vec2 {
        self.tilt
    }

    #[must_use]
    pub fn get_tilt_mut(&mut self) -> &mut Vec2 {
        &mut self.tilt
    }
}

impl Material2d for TileMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}
