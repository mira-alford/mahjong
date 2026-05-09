pub mod kind;
pub mod render;

use bevy::prelude::*;
use std::time::Instant;

use self::kind::{Suit, TileKind};
use self::render::{TileMaterial, TileMaterialPlugin};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TileMaterialPlugin {})
            .add_systems(Update, (lerp_tiles, update_tile_materials));
    }
}

#[derive(Component, Debug)]
struct Tile {
    data: TileKind,
}

/// material for a tiles face
#[derive(Component)]
struct TileFaceMaterial(Handle<TileMaterial>);

/// material for tiles face
#[derive(Component)]
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
    mesh: &Handle<Mesh>,
    materials: &mut ResMut<Assets<TileMaterial>>,
    asset_server: AssetServer,
) -> Entity {
    let face_material = materials.add(TileMaterial::new(
        asset_server.load("front-face-placeholder.png"),
    ));
    let back_material = materials.add(TileMaterial::new(
        asset_server.load("back-face-placeholder.png"),
    ));

    commands
        .spawn((
            TileFaceMaterial(face_material.clone()),
            TileBackMaterial(back_material),
            ShownFace::default(),
            Transform::default().with_scale(Vec3::splat(128.0)),
            Tile {
                data: TileKind::Suit(Suit::Characters(1)),
            },
            // TODO: this should probably be done with a resource specified when plugin made
            Mesh2d(mesh.clone()),
            MeshMaterial2d(face_material),
        ))
        .observe(tile_click_oberver)
        .observe(tile_hover_observer)
        .observe(tile_unhover_observer)
        .id()
}

fn update_tile_materials(
    mut query: Query<
        (
            &ShownFace,
            &TileFaceMaterial,
            &TileBackMaterial,
            &mut MeshMaterial2d<TileMaterial>,
        ),
        Changed<ShownFace>,
    >,
) {
    for (face, top_material, back_material, mut material) in query.iter_mut() {
        match face.0 {
            TileFace::Top => material.0 = top_material.0.clone(),
            TileFace::Bottom => material.0 = back_material.0.clone(),
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

fn tile_hover_observer(
    event: On<Pointer<Over>>,
    query: Query<&MeshMaterial2d<TileMaterial>>,
    mut materials: ResMut<Assets<TileMaterial>>,
) {
    let event_target = event.event_target();

    let material_handle = query
        .get(event_target)
        .expect("hovered tile did not have a meshmaterial2d componenet");

    let material = materials.get_mut(material_handle).unwrap(); // probably impossible to hit this unwrap

    material.set_tint(Color::linear_rgba(0.7, 0.7, 0.7, 1.0))
}

fn tile_unhover_observer(
    event: On<Pointer<Out>>,
    query: Query<&MeshMaterial2d<TileMaterial>>,
    mut materials: ResMut<Assets<TileMaterial>>,
) {
    let event_target = event.event_target();

    let material_handle = query
        .get(event_target)
        .expect("hovered tile did not have a meshmaterial2d componenet");

    let material = materials.get_mut(material_handle).unwrap(); // probably impossible to hit this unwrap

    material.set_tint(Color::WHITE)
}

fn stretched_exp(x: f32, a: f32, b: f32) -> f32 {
    1.0 - (-(x / a).powf(b)).exp()
}

fn lerp_tiles(mut commands: Commands, mut tiles: Query<(Entity, &MoveCurve, &mut Transform)>) {
    let now = Instant::now();
    for (entity, curve, mut transform) in &mut tiles {
        let delta = now.duration_since(curve.start_time).as_secs_f32();
        let move_scalar = stretched_exp(delta, curve.a, curve.b);
        let new_pos: Vec2 = curve.start + move_scalar * (curve.end - curve.start);
        transform.translation = new_pos.extend(0.0);

        if 1.0 - move_scalar < 1e-5 {
            commands.entity(entity).remove::<MoveCurve>();
        }
    }
}
