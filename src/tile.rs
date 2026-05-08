mod render;

use bevy::prelude::*;
use std::time::Instant;

use self::render::{TileMaterial, TileMaterialPlugin};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TileMaterialPlugin {})
            .add_systems(Startup, setup)
            .add_systems(Update, (lerp_tiles, update_tile_materials, layout_hand));
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

    let tile_face_material = materials.add(TileMaterial::new(
        asset_server.load("front-face-placeholder.png"),
    ));

    // 4.0 / 3.0 because the tiles are 3:4 aspect ratio
    let tile_mesh = meshes.add(Rectangle::from_size(Vec2::new(1.0, 4.0 / 3.0)));

    let hand_id = spawn_hand(&mut commands);

    let tile_id = spawn_tile(&mut commands, &tile_face_material, &tile_mesh);
    commands.entity(tile_id).insert(OwnedTile(hand_id)); // i.e., tile is 'owned' by hand
    let tile_id = spawn_tile(&mut commands, &tile_face_material, &tile_mesh);
    commands.entity(tile_id).insert(OwnedTile(hand_id)); // i.e., tile is 'owned' by hand
    let tile_id = spawn_tile(&mut commands, &tile_face_material, &tile_mesh);
    commands.entity(tile_id).insert(OwnedTile(hand_id)); // i.e., tile is 'owned' by hand
    let tile_id = spawn_tile(&mut commands, &tile_face_material, &tile_mesh);
    commands.entity(tile_id).insert(OwnedTile(hand_id)); // i.e., tile is 'owned' by hand
}

/// Vec2 denoting the position of where the hand should be rendered and a float length?
#[derive(Component, Debug)]
struct HandAnchor(Vec2, f32);

/// Vec2 denoting the position of where the discord pile should be rendered
#[derive(Component, Debug)]
struct DiscardAnchor(Vec2);

/// Relationship that points from the tile to the 'owner' hand
#[derive(Component, Debug)]
#[relationship(relationship_target = TileCollection)]
struct OwnedTile(pub Entity);

/// Relationship denoting the hand that holds all of the tiles
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = OwnedTile, linked_spawn)]
struct TileCollection(Vec<Entity>);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TileKind {
    Suit(Suit),
    Honor(Honor),
}

/// suits each with associated number between 1 and 9
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    Characters(u8),
    Circle(u8),
    Bamboo(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Honor {
    Wind(Wind),
    Dragon(Dragon),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dragon {
    Red,
    Green,
    White,
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
struct MoveCurve {
    start: Vec2,
    end: Vec2,
    start_time: Instant,
    a: f32,
    b: f32,
}

/// Spawner of the hand
pub fn spawn_hand(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            HandAnchor(Vec2::new(-500.0, -300.0), 1000.0),
            TileCollection::default(),
            Transform::default().with_scale(Vec3::splat(128.0)),
        ))
        .id()
}

/// spawns a tile with a specified front facing material, and a mesh
fn spawn_tile(
    commands: &mut Commands,
    material: &Handle<TileMaterial>,
    mesh: &Handle<Mesh>,
) -> Entity {
    commands
        .spawn((
            TileFaceMaterial(material.clone()),
            ShownFace::default(),
            MoveCurve {
                start: Vec2::ZERO,
                end: Vec2::new(500.0, 500.0),
                start_time: Instant::now(),
                a: 1.0,
                b: 3.5,
            },
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

/// `a` and `b` params that are used in our move curve functions
/// these dictate the way in which tiles are moved (in terms of speed)
/// for laying out a hand
const LAYOUT_HAND_MOVE_A: f32 = 1.0;
const LAYOUT_HAND_MOVE_B: f32 = 3.5;

/// Goes through the hand collections that have a hand anchor and puts the appropriate [`MoveCurve`]
/// on the tile based on where it needs to go relative to the [`HandAnchor`].
fn layout_hand(
    mut commands: Commands,
    hand_anchors: Query<(Entity, &HandAnchor)>,
    all_tiles: Query<(&Transform, Option<&MoveCurve>)>,
    tile_collections: Query<&TileCollection>,
) {
    for (hand_entity, HandAnchor(anchor_pos, anchor_len)) in hand_anchors {
        let tile_iter: Vec<_> = tile_collections.iter_descendants(hand_entity).collect();

        // collect all of the tiles that we own (filtering out non-tiles)
        for (i, tile) in tile_iter.iter().enumerate() {
            // we always add offset regardless because some entities might be filling slots
            // (e.g., placeholder tile that we don't render but still affects offset)
            let cur_offset = i as f32 * anchor_len / tile_iter.len() as f32;

            let Ok((tile_transform, opt_move_curve)) = all_tiles.get(*tile) else {
                continue; // if not owned, we skip
            };

            // calculate where tile should be
            let new_tile_pos = anchor_pos + Vec2::X * cur_offset;
            let existing_tile_pos = tile_transform.translation.xy();

            let pos_delta = (existing_tile_pos - new_tile_pos).length();

            // if position change is super small, don't bother moving
            if pos_delta < 1e-4 {
                continue;
            }

            // @Jackson, can you fix this :)))))))
            if let Some(move_curve) = opt_move_curve {
                let existing_tile_pos = move_curve.end;

                let pos_delta = (existing_tile_pos - new_tile_pos).length();

                // if position change is super small, don't bother moving
                if pos_delta < 1e-4 {
                    continue;
                }
            }

            let move_curve = MoveCurve {
                start: existing_tile_pos,
                end: new_tile_pos,
                start_time: Instant::now(),
                a: LAYOUT_HAND_MOVE_A,
                b: LAYOUT_HAND_MOVE_B,
            };

            commands.entity(*tile).insert(move_curve);
        }
    }
}
