pub mod kind;
pub mod render;

use bevy::prelude::*;
use std::default;
use std::time::Instant;

use crate::layout::{LAYOUT_HAND_MOVE_A, LAYOUT_HAND_MOVE_B};
use crate::level::Owner;

use self::kind::{Suit, TileKind};
use self::render::{TileMaterial, TileMaterialPlugin};

pub struct TilePlugin;

pub const TILE_WIDTH: f32 = 96.0;
pub const TILE_HEIGHT: f32 = 128.0;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TileMaterialPlugin {})
            .add_systems(Startup, SharedTileData::init_system)
            .add_systems(Update, lerp_tiles)
            .add_systems(FixedPostUpdate, (move_tile, rotate_tile))
            .add_message::<MoveTile>()
            .add_message::<RotateTile>();
    }
}

#[derive(Message)]
pub struct MoveTile {
    pub id: Entity,
    pub dest: Vec2,
}

#[derive(Message)]
pub struct RotateTile {
    pub id: Entity,
    pub owner: Owner,
}

#[derive(Component, Debug)]
struct Tile {
    kind: TileKind,
}

#[derive(Bundle)]
pub struct TileBundle {
    tile: Tile,
    mesh: Mesh2d,
    material: MeshMaterial2d<TileMaterial>,
    shown_face: ShownFace,
    transform: Transform,
}

/// contains data shared between tiles, like their base front image, back image
#[derive(Resource, Clone)]
pub struct SharedTileData {
    mesh: Handle<Mesh>,
    front_texture: Handle<Image>,
    back_texture: Handle<Image>,
}

impl SharedTileData {
    fn new(meshes: &mut Assets<Mesh>, asset_server: AssetServer) -> Self {
        Self {
            mesh: meshes.add(Rectangle::from_size(Vec2::new(TILE_WIDTH, TILE_HEIGHT))), // 3:4 aspect ratio on tiles
            front_texture: asset_server.load("sprites/tiles/Front.png"),
            back_texture: asset_server.load("sprites/tiles/Back.png"),
        }
    }

    fn init_system(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        asset_server: Res<AssetServer>,
    ) {
        commands.insert_resource(Self::new(&mut meshes, asset_server.clone()));
    }
}

impl TileBundle {
    pub fn new(
        materials: &mut Assets<TileMaterial>,
        asset_server: AssetServer,
        shared_data: SharedTileData,
        kind: TileKind,
    ) -> Self {
        let load_sprite = |file: &str| {
            Some((|file: &str| {
                asset_server.load::<Image>("sprites/tiles/".to_string() + file)
            })(file))
        };

        let load_suit_sprite = |number: u8, file_prefix: &str| {
            if number > 9 {
                error!(
                    "somone has tried to make a tile with a number above 9 and i cant get a texture for it :("
                )
            }

            load_sprite(&format!("{file_prefix}{number}.png"))
        };

        let overlay_texture = match kind {
            TileKind::Number(suit, num) => match suit {
                Suit::Characters => load_suit_sprite(num, "Man"),
                Suit::Circle => load_suit_sprite(num, "Pin"),
                Suit::Bamboo => load_suit_sprite(num, "Sou"),
            },
            TileKind::Honor(honor) => match honor {
                kind::Honor::Wind(wind) => match wind {
                    kind::Wind::East => load_sprite("Ton.png"),
                    kind::Wind::South => load_sprite("Nan.png"),
                    kind::Wind::West => load_sprite("Shaa.png"),
                    kind::Wind::North => load_sprite("Pei.png"),
                },
                kind::Honor::Dragon(dragon) => match dragon {
                    kind::Dragon::Red => load_sprite("Chun.png"),
                    kind::Dragon::Green => load_sprite("Hatsu.png"),
                    kind::Dragon::White => load_sprite("Haku.png"),
                },
            },
            TileKind::Blank => load_sprite("Blank.png"),
        };

        let material = MeshMaterial2d(materials.add(TileMaterial::new(
            shared_data.front_texture,
            shared_data.back_texture,
            overlay_texture,
        )));

        Self {
            tile: Tile { kind },
            mesh: Mesh2d(shared_data.mesh),
            material: material,
            shown_face: ShownFace::default(),
            transform: Transform::default(),
        }
    }
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

fn move_tile(
    mut messages: MessageReader<MoveTile>,
    mut commands: Commands,
    query: Query<(&Transform, Option<&MoveCurve>)>,
) {
    for &MoveTile { id, dest } in messages.read() {
        let Ok((transform, curve)) = query.get(id) else {
            continue;
        };

        let existing_tile_pos = transform.translation.xy();
        let pos_delta = (existing_tile_pos - dest).length();

        // if position change is super small, don't bother moving
        if pos_delta < 1e-4 {
            continue;
        }

        let mut time = Instant::now();
        if let Some(curve) = curve {
            let existing_tile_pos = curve.end;
            let pos_delta = (existing_tile_pos - dest).length();

            // if position change is super small, don't bother moving
            if pos_delta < 1e-4 {
                continue;
            }

            // It is also relevant, if the new move curve is in the direction
            // of the old curve (roughly) then we keep the same time/velocity.
            if dest.dot(curve.end) > 0.8 {
                time = curve.start_time;
            }
        }

        // Otherwise, add a move curve
        let move_curve = MoveCurve {
            start: existing_tile_pos,
            end: dest,
            start_time: time,
            a: LAYOUT_HAND_MOVE_A,
            b: LAYOUT_HAND_MOVE_B,
        };
        commands.entity(id).insert(move_curve);
    }
}

/// rotates the tile 180 degrees
fn rotate_tile(mut messages: MessageReader<RotateTile>, mut query: Query<&mut Transform>) {
    for &RotateTile { id, owner } in messages.read() {
        if let Ok(mut transform) = query.get_mut(id) {
            *transform = transform.with_rotation(Quat::from_rotation_z(match owner {
                Owner::AI => core::f32::consts::TAU / 2f32,
                Owner::Player => 0f32,
            }));
        }
    }
}
