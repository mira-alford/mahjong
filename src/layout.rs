use std::time::Instant;

use bevy::prelude::*;
use itertools::Itertools;
use rand::distr::{Distribution, Uniform};
use rand::rngs::{SmallRng, StdRng};
use rand::seq::{IndexedRandom, SliceRandom};
use rand::{RngExt, SeedableRng};

use crate::tile::MoveCurve;

pub fn layout_plugin(app: &mut App) {
    app.add_systems(FixedUpdate, layout_hand)
        .add_systems(FixedUpdate, transfer_tiles)
        .add_message::<TransferTile>();
}

/// Vec2 denoting the position of where the hand should be rendered and a float length?
#[derive(Component, Debug)]
pub struct HandAnchor(pub Vec2, pub f32);

/// Vec2 denoting the width/height of the walls rect (from the center).
/// IVec2 denoting the number of tiles on the x and y
#[derive(Component, Debug)]
pub struct WallAnchor(pub Vec2, pub IVec2);

/// Vec2 denoting the position of where the discord pile should be rendered
#[derive(Component, Debug)]
pub struct DiscardAnchor(pub Vec2);

/// All the tiles atop eachother in a glorious heap.
#[derive(Component, Debug)]
pub struct UnusedAnchor(pub Vec2);

/// Relationship that points from the tile to the 'owner' hand
#[derive(Component, Debug)]
#[relationship(relationship_target = TileCollection)]
pub struct OwnedTile(pub Entity);

/// Relationship denoting the hand that holds all of the tiles
#[derive(Component, Debug, Default)]
#[relationship_target(relationship = OwnedTile, linked_spawn)]
pub struct TileCollection(Vec<Entity>);

#[derive(Component, Debug)]
pub struct Slot(pub u8);

/// `a` and `b` params that are used in our move curve functions
/// these dictate the way in which tiles are moved (in terms of speed)
/// for laying out a hand
const LAYOUT_HAND_MOVE_A: f32 = 1.0;
const LAYOUT_HAND_MOVE_B: f32 = 3.5;

#[derive(Message)]
pub struct TransferTile {
    pub tile: Entity,
    pub src: Entity,
    pub dest: Entity,
}

/// Event handler for transferring tiles from one collection to another.
fn transfer_tiles(mut messages: MessageReader<TransferTile>, mut commands: Commands) {
    for &TransferTile { tile, src, dest } in messages.read() {
        commands.entity(src).remove_related::<OwnedTile>(&[tile]);
        commands.entity(dest).add_one_related::<OwnedTile>(tile);
    }
}

/// Goes through the hand collections that have a hand anchor and puts the appropriate [`MoveCurve`]
/// on the tile based on where it needs to go relative to the [`HandAnchor`].
fn layout_hand(
    mut commands: Commands,
    hand_anchors: Query<(Entity, &HandAnchor)>,
    all_tiles: Query<(&Transform, Option<&MoveCurve>, &Slot)>,
    tile_collections: Query<&TileCollection>,
) {
    for (hand_entity, HandAnchor(anchor_pos, anchor_len)) in hand_anchors {
        let tile_iter: Vec<_> = tile_collections.iter_descendants(hand_entity).collect();

        // collect all of the tiles that we own (filtering out non-tiles)
        for tile in tile_iter.iter() {
            // we always add offset regardless because some entities might be filling slots
            // (e.g., placeholder tile that we don't render but still affects offset)

            let Ok((tile_transform, opt_move_curve, curslot)) = all_tiles.get(*tile) else {
                continue; // if not owned, we skip
            };
            let cur_offset = curslot.0 as f32 / 14.0 * anchor_len;

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

fn layout_wall(
    wall_anchor: Query<(Entity, &WallAnchor)>,
    all_tiles: Query<(&Transform, Option<&MoveCurve>)>,
    tile_collections: Query<&TileCollection>,
) {
    let mut rng = StdRng::seed_from_u64(67); // -\_o_o_/^
    let Ok((wall_entity, wall_anchor)) = wall_anchor.single() else {
        error!("Expected 1 wall anchor");
        return;
    };

    let mut positions = (0..(wall_anchor.1.x))
        .map(|i| {
            i as f32 * wall_anchor.0.x as f32 / wall_anchor.1.x as f32 - (wall_anchor.0.x / 2.0)
        })
        .cartesian_product((0..wall_anchor.1.y).map(|i| {
            i as f32 * wall_anchor.0.y as f32 / wall_anchor.1.y as f32 - (wall_anchor.0.y / 2.0)
        }))
        .map(|(x, y)| Vec2::new(x, y))
        .collect_vec();

    positions.shuffle(&mut rng);
    let mut i = 0;

    for tile_entity in tile_collections.iter_descendants(wall_entity) {
        let pos = positions.get(i).unwrap();
        i = (i + 1) % positions.len();
    }
}
