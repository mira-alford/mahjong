use bevy::{
    picking::hover::Hovered,
    prelude::*,
    render::render_resource::AsBindGroup,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
};

use crate::tile::ShownFace;

const SHADER_PATH: &str = "shaders/tile_shader.wgsl";

pub struct TileMaterialPlugin;

impl Plugin for TileMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<TileMaterial>::default())
            .add_systems(
                Update,
                (start_flip_animation, flip_animation, hover_animation).chain(),
            );
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
    #[uniform(7)]
    flags: u32, // only flag is bit 1, on for face down off for face up
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
            flags: 0,
        }
    }

    // tint setters and getters
    pub fn set_tint(&mut self, tint: impl Into<LinearRgba>) {
        self.tint = tint.into();
    }

    pub fn get_tint(&self) -> LinearRgba {
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

    pub fn get_scale(&self) -> Vec2 {
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

    pub fn get_tilt(&self) -> Vec2 {
        self.tilt
    }

    #[must_use]
    pub fn get_tilt_mut(&mut self) -> &mut Vec2 {
        &mut self.tilt
    }

    // texture card flip spawpping getters and setters
    pub fn set_flipped(&mut self, flip: bool) {
        match flip {
            true => self.flags = 1,
            false => self.flags = 0,
        }
    }
}

impl Material2d for TileMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_PATH.into()
    }

    fn vertex_shader() -> bevy::shader::ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> bevy::sprite_render::AlphaMode2d {
        bevy::sprite_render::AlphaMode2d::Blend
    }
}

/// marker componenet for [`Tile`]s that should react to hovering
#[derive(Component)]
pub struct HoveringAnimation;

fn hover_animation(
    time: Res<Time>,
    mut materials: ResMut<Assets<TileMaterial>>,
    material_query: Query<(&mut MeshMaterial2d<TileMaterial>, &Hovered), With<HoveringAnimation>>,
) {
    for (material_handle, hovered) in material_query.iter() {
        let target = if hovered.0 {
            Vec2::new(1.2, 1.2)
        } else {
            Vec2::new(1.0, 1.0)
        };

        let material = materials.get_mut(&material_handle.0).expect("darn");
        let curr_scale = material.clone().get_scale();

        let lerp_factor = 10.0 * time.delta_secs();

        material.set_scale(&curr_scale.lerp(target, lerp_factor));

        if hovered.0 {
            material.set_tilt(&Vec2::new(0.0, 30.0));
        } else {
            material.set_tilt(&Vec2::ZERO)
        }
    }
}

// process for flipping:
// somone changes ShownFace on a tile
// flip animation animates the change, and actually changes the texture half way thru the animation

/// componenet to mark things undergoing a flipping animation, and holding state
#[derive(Component, Default)]
pub struct FlipInProgress {
    t: f32,
    // if we have told the shader to swap texture yet
    done_flip: bool,
}

fn start_flip_animation(
    mut commands: Commands,
    query: Query<Entity, Changed<ShownFace>>,
    mut first_run: Local<bool>,
) {
    // changed is triggered on the first run so need to skip first one
    if !*first_run {
        *first_run = true;
        return;
    }
    for entity in query.iter() {
        commands
            .get_entity(entity)
            .expect("ruh rho")
            .insert(FlipInProgress::default());
    }
}

// outputs the value of the x scaling factor for different values of t
fn flip_scale_function(t: f32) -> f32 {
    (4.0 * (t - 0.5).powi(2)).min(1.0)
}

fn flip_animation(
    mut commands: Commands,
    mut materials: ResMut<Assets<TileMaterial>>,
    time: Res<Time>,
    query: Query<(
        Entity,
        &ShownFace,
        &mut FlipInProgress,
        &MeshMaterial2d<TileMaterial>,
    )>,
) {
    for (entity, shown_face, mut flip_animation_state, material_handle) in query {
        let material = materials
            .get_mut(&material_handle.0)
            .expect("uhhh what the sigma");

        let x_scale_factor = flip_scale_function(flip_animation_state.t);
        let y_scale_factor = material.get_scale().y;

        // since animation ends when t == 1, the animation will last 0.5 second
        flip_animation_state.t += time.delta_secs() * 2.0;

        // dbg!(flip_animation_state.t);
        // dbg!(x_scale_factor);

        if 0.5 <= flip_animation_state.t && !flip_animation_state.done_flip {
            match shown_face.0 {
                super::TileFace::Top => material.set_flipped(false),
                super::TileFace::Bottom => material.set_flipped(true),
            }
        }

        if flip_animation_state.t >= 1.0 {
            material.set_scale(&Vec2::new(1.0, y_scale_factor));
            commands
                .get_entity(entity)
                .expect("HHHHHHH")
                .remove::<FlipInProgress>();
            return;
        }

        material.set_scale(&Vec2::new(x_scale_factor, y_scale_factor));
    }
}
