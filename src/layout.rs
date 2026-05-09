use bevy::prelude::*;
use itertools::Itertools;
use std::ops::Neg;

use crate::level::Owner;
use crate::tile::{MoveTile, RotateTile, TILE_HEIGHT, TILE_WIDTH};

pub fn layout_plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            layout_hand,
            layout_wall,
            layout_discard,
            // resize_wall,
            // resize_hand,
        ),
    )
    .add_systems(FixedUpdate, transfer_tiles)
    .add_message::<TransferTile>()
    .add_message::<RotateTile>();
}

/// Vec2 denoting the position of where the hand should be rendered and a float length?
#[derive(Component, Debug)]
pub struct HandAnchor(pub Vec2, pub Owner);

/// IVec2 denoting the number of tiles on the x and y
#[derive(Component, Debug)]
pub struct WallAnchor(pub IVec2);

/// Vec2 denoting the position of where the discord pile should be rendered
/// DiscardAnchor.1 is the maximum width in tile count for discard layouting
#[derive(Component, Debug)]
pub struct DiscardAnchor(pub Vec2, pub u8, pub Owner);

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
pub const LAYOUT_HAND_MOVE_A: f32 = 1.0;
pub const LAYOUT_HAND_MOVE_B: f32 = 3.5;

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
    hand_anchors: Query<(Entity, &HandAnchor)>,
    tile_collections: Query<&TileCollection>,
    all_tiles: Query<&Slot>,
    mut move_tiles_writer: MessageWriter<MoveTile>,
    mut flip_tiles_writer: MessageWriter<RotateTile>,
) {
    for (hand_entity, &HandAnchor(anchor_pos, owner)) in hand_anchors {
        let tile_iter: Vec<_> = tile_collections.iter_descendants(hand_entity).collect();

        // collect all of the tiles that we own (filtering out non-tiles)
        for tile in tile_iter.iter() {
            let Ok(curslot) = all_tiles.get(*tile) else {
                continue; // if not owned, we skip
            };

            let mut cur_offset = curslot.0 as f32 * TILE_WIDTH;
            if curslot.0 == 13 {
                cur_offset += TILE_WIDTH;
            }

            let new_tile_pos = match owner {
                Owner::AI => anchor_pos - Vec2::X * cur_offset,
                Owner::Player => anchor_pos + Vec2::X * cur_offset,
            };

            move_tiles_writer.write(MoveTile {
                id: *tile,
                dest: new_tile_pos,
            });

            flip_tiles_writer.write(RotateTile { id: *tile, owner });
        }
    }
}

// todo:
// - layoutdiscard function: similar to layout wall. Should check what tiles are currently in discard and all tiles and the relationship between tiles and collections
// for each discard, for each tile belonging to that discard;
//
// starts at position of the loops discard anchor, separates right by a tile width each time,
// wraps around when it reaches the edge

fn layout_discard(
    discard_anchors: Query<(Entity, &DiscardAnchor)>,
    tile_collections: Query<&TileCollection>,
    mut move_tiles_writer: MessageWriter<MoveTile>,
    mut flip_tiles_writer: MessageWriter<RotateTile>,
) {
    for (discard_entity, &DiscardAnchor(anchorpos, discard_layout_width, player_kind)) in
        discard_anchors
    {
        let (width, height) = match player_kind {
            // todo flip the ai's (playerside up) cards upside down (requires transform stuff)
            Owner::Player => (TILE_WIDTH, TILE_HEIGHT.neg()),
            Owner::AI => (TILE_WIDTH.neg(), TILE_HEIGHT),
        };

        for (ix, tile) in tile_collections
            .iter_descendants(discard_entity)
            .enumerate()
        {
            let new_pos = anchorpos
                + Vec2::new(
                    // could div_euclid instead
                    (ix % (discard_layout_width as usize)) as f32 * width,
                    (ix / (discard_layout_width as usize)) as f32 * height,
                );

            move_tiles_writer.write(MoveTile {
                id: tile,
                dest: new_pos,
            });

            flip_tiles_writer.write(RotateTile {
                id: tile,
                owner: player_kind,
            });
        }
    }
}

fn layout_wall(
    wall_anchor: Query<(Entity, &WallAnchor)>,
    tile_collections: Query<&TileCollection>,
    all_tiles: Query<&Slot>,
    mut move_tiles_writer: MessageWriter<MoveTile>,
) {
    let Some((wall_entity, wall_anchor)) = wall_anchor.iter().next() else {
        return;
    };

    let dims = wall_anchor.0.as_vec2() * Vec2::new(TILE_WIDTH, TILE_HEIGHT);

    let top = (0..=(wall_anchor.0.x))
        .map(|i| i as f32 * TILE_WIDTH)
        .map(|x| Vec2::new(x, dims.y) - dims / 2.0)
        .collect_vec();
    let bottom = (0..=(wall_anchor.0.x))
        .rev()
        .map(|i| i as f32 * TILE_WIDTH)
        .map(|x| Vec2::new(x, 0.0) - dims / 2.0)
        .collect_vec();
    let right = (0..=(wall_anchor.0.y))
        .map(|i| i as f32 * TILE_HEIGHT)
        .map(|y| Vec2::new(dims.x, y) - dims / 2.0)
        .collect_vec();
    let left = (0..=(wall_anchor.0.y))
        .rev()
        .map(|i| i as f32 * TILE_HEIGHT)
        .map(|y| Vec2::new(0.0, y) - dims / 2.0)
        .collect_vec();

    let positions = [top, right, bottom, left].concat();

    for tile_entity in tile_collections.iter_descendants(wall_entity) {
        let Ok(slot) = all_tiles.get(tile_entity) else {
            continue;
        };
        let pos = positions.get(slot.0 as usize % positions.len()).unwrap();

        move_tiles_writer.write(MoveTile {
            id: tile_entity,
            dest: *pos,
        });
    }
}

// fn camera_scaling(
//     camera: Single<(&mut Camera2d, &mut Viewport)>,
//     mut walls: Query<&mut WallAnchor>,
// ) {
//     let Ok(mut wall_anchor) = walls.single_mut() else {
//         return;
//     };
// }

// fn resize_hand(window: Single<&Window>, mut hands: Query<(&mut HandAnchor, &Owner)>) {
//     for (mut hand_anchor, owner) in &mut hands {
//         match owner {
//             Owner::Player => {
//                 hand_anchor.0 = window.size() * Vec2::new(-1.0, 1.0) / 2.0
//                     + Vec2::new(TILE_WIDTH * 4.5, TILE_HEIGHT * 7.5)
//             }
//             Owner::AI => {
//                 hand_anchor.0 = window.size() * Vec2::new(1.0, -1.0) / 2.0
//                     + Vec2::new(TILE_WIDTH * 4.5, TILE_HEIGHT * 7.5)
//             }
//         }
//     }
// }
