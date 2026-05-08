pub mod render;

use bevy::prelude::*;
use std::time::Instant;

use self::render::{TileMaterial, TileMaterialPlugin};
use crate::tile_kind::{Suit, TileKind};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TileMaterialPlugin {})
            .add_systems(Startup, setup)
            .add_systems(Update, (lerp_tiles, update_tile_materials));
    }
}

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TileMaterial>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // TODO: make this editiable on the plugin
    commands.insert_resource(TileBackMaterial(materials.add(TileMaterial::new(
        asset_server.load("back-face-placeholder.png"),
    ))));
}

#[derive(Component, Debug)]
struct Tile {
    data: TileKind,
}

/// material for a tiles face
#[derive(Component)]
struct TileFaceMaterial(Handle<TileMaterial>);

// Shared resource for the back of tiles (since all tiles are same on back)
#[derive(Resource)]
struct TileBackMaterial(Handle<TileMaterial>);

/// the currently up facing face of a tile, i.e. the face you can see
#[derive(Component, Default)]
struct ShownFace(TileFace);

#[derive(Default)]
enum TileFace {
    #[default]
    Top,
    Bottom,
}

/// movement curve
#[derive(Component, Debug)]
pub struct MoveCurve {
    pub start: Vec2,
    pub end: Vec2,
    pub start_time: Instant,
    pub a: f32,
    pub b: f32,
}

/// spawns a tile with a specified front facing material, and a mesh
pub fn spawn_tile(
    commands: &mut Commands,
    material: &Handle<TileMaterial>,
    mesh: &Handle<Mesh>,
) -> Entity {
    commands
        .spawn((
            TileFaceMaterial(material.clone()),
            ShownFace::default(),
            Transform::default().with_scale(Vec3::splat(128.0)),
            Tile {
                data: TileKind::Suit(Suit::Characters(1)),
            },
            // TODO: this should probably be done with a resource specified when plugin made
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
        ))
        .observe(tile_click_oberver)
        .id()
}

fn update_tile_materials(
    mut query: Query<
        (
            &ShownFace,
            &TileFaceMaterial,
            &mut MeshMaterial2d<TileMaterial>,
        ),
        Changed<ShownFace>,
    >,
    bottom_material: Res<TileBackMaterial>,
) {
    for (face, top_material, mut material) in query.iter_mut() {
        match face.0 {
            TileFace::Top => material.0 = top_material.0.clone(),
            TileFace::Bottom => material.0 = bottom_material.0.clone(),
        }
    }
}

fn tile_click_oberver(event: On<Pointer<Click>>, mut query: Query<&mut ShownFace>) {
    let event_target = event.event_target();

    println!("clicked {:?}", event_target);

    let mut face = query
        .get_mut(event_target)
        .expect("expected clicked tile to have ShownFace componenet");

    match face.0 {
        TileFace::Top => face.0 = TileFace::Bottom,
        TileFace::Bottom => face.0 = TileFace::Top,
    }
}

fn stretched_exp(x: f32, a: f32, b: f32) -> f32 {
    1.0 - (-(x / a).powf(b)).exp()
}

fn lerp_tiles(mut commands: Commands, mut tiles: Query<(Entity, &MoveCurve, &mut Transform)>) {
    for (entity, curve, mut transform) in &mut tiles {
        let now = Instant::now();
        let delta = now.duration_since(curve.start_time).as_secs_f32();
        let move_scalar = stretched_exp(delta, curve.a, curve.b);
        let new_pos: Vec2 = curve.start + move_scalar * (curve.end - curve.start);
        transform.translation = new_pos.extend(0.0);

        if 1.0 - move_scalar < 1e-5 {
            commands.entity(entity).remove::<MoveCurve>();
        }
    }
}
