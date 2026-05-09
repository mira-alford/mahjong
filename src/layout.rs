use bevy::prelude::*;
use itertools::Itertools;
use std::ops::Neg;

use crate::level::Owner;
use crate::model::game::GameModel;
use crate::tile::{MoveTile, RotateTile, ShownFace, TILE_HEIGHT, TILE_WIDTH, Tile};

/// Hand Anchor
#[derive(Copy, Debug, Clone)]
pub struct Hand {
    /// Vec2 denoting the position of where the hand should be rendered and a float length?
    pub pos: Vec2,
}

/// Wall Anchor
#[derive(Copy, Debug, Clone)]
pub struct Wall {
    /// IVec2 denoting the number of tiles on the x and y
    pub pos: IVec2,
}

/// Discard Pile Anchor
#[derive(Copy, Debug, Clone)]
pub struct Discard {
    /// denotes the position of where the discord pile should be rendered
    pub pos: Vec2,
    /// maximum width in tile count for discard layouting
    pub max_width: u8,
}

/// All the tiles atop eachother in a glorious heap.
#[derive(Copy, Debug, Clone)]
pub struct Unused;

/// Where you've drawn your tile to in your hand
#[derive(Copy, Debug, Clone)]
pub struct Draw(pub Vec2);

#[derive(Component, Copy, Clone, Debug)]
pub enum Anchor {
    Hand(Hand),
    Wall(Wall),
    Discard(Discard),
    Unused(Unused),
    Draw(Draw),
}

pub fn layout_plugin(app: &mut App) {
    app.add_systems(FixedUpdate, layout_all_the_things)
        .add_systems(FixedUpdate, transfer_tiles)
        // .add_systems(FixedUpdate, flip_hidden_tiles)
        .add_message::<TransferTile>()
        .add_message::<RotateTile>()
        .insert_resource(ClearColor(Color::srgb_u8(60, 153, 111)));
    // 46cc8c
}

fn layout_all_the_things(
    anchor_query: Query<(Entity, &Anchor, Option<&Owner>)>,
    tile_collections: Query<&TileCollection>,
    tiles_query: Query<&Tile>,
    mut move_tiles_writer: MessageWriter<MoveTile>,
    mut flip_tiles_writer: MessageWriter<RotateTile>,
) {
    for (anchor_entity, anchor, owner_opt) in anchor_query {
        let tiles: Vec<_> = tile_collections.iter_descendants(anchor_entity).collect();

        match anchor {
            Anchor::Hand(hand) => layout_hand(
                *hand,
                *owner_opt.expect("every hand anchor should also have a owner"),
                tiles
                    .iter()
                    .map(|e| (*e, tiles_query.get(*e).expect("be civil")))
                    .collect(),
                &mut move_tiles_writer,
                &mut flip_tiles_writer,
            ),
            Anchor::Wall(wall) => layout_wall(*wall, tiles, &mut move_tiles_writer),
            Anchor::Discard(discard) => layout_discard(
                *discard,
                *owner_opt.expect("every discard anchor should also have a owner"),
                tiles,
                &mut move_tiles_writer,
                &mut flip_tiles_writer,
            ),
            Anchor::Unused(_) => {}
            Anchor::Draw(_) => {}
        };
    }
}

/// Goes through the hand collections that have a hand anchor and puts the appropriate [`MoveCurve`]
/// on the tile based on where it needs to go relative to the [`HandAnchor`].
fn layout_hand(
    hand: Hand,
    owner: Owner,
    tiles: Vec<(Entity, &Tile)>,
    move_tiles_writer: &mut MessageWriter<MoveTile>,
    flip_tiles_writer: &mut MessageWriter<RotateTile>,
) {
    // collect all of the tiles that we own (filtering out non-tiles)
    for (i, (tile_entity, tile)) in tiles
        .iter()
        .sorted_by_key(|(tile_entity, tile)| tile.kind)
        .enumerate()
    {
        let mut cur_offset = i as f32 * TILE_WIDTH;
        if i == 13 {
            cur_offset += TILE_WIDTH;
        }

        let new_tile_pos = match owner {
            Owner::AI => hand.pos - Vec2::X * cur_offset,
            Owner::Player => hand.pos + Vec2::X * cur_offset,
        };

        move_tiles_writer.write(MoveTile {
            id: *tile_entity,
            dest: new_tile_pos,
        });

        flip_tiles_writer.write(RotateTile {
            id: *tile_entity,
            owner,
        });
    }
}

fn layout_discard(
    discard: Discard,
    owner: Owner,
    tiles: Vec<Entity>,
    move_tiles_writer: &mut MessageWriter<MoveTile>,
    flip_tiles_writer: &mut MessageWriter<RotateTile>,
) {
    let (width, height) = match owner {
        // todo flip the ai's (playerside up) cards upside down (requires transform stuff)
        Owner::Player => (TILE_WIDTH, TILE_HEIGHT.neg()),
        Owner::AI => (TILE_WIDTH.neg(), TILE_HEIGHT),
    };

    for (idx, tile) in tiles.iter().enumerate() {
        let new_pos = discard.pos
            + Vec2::new(
                // could div_euclid instead
                (idx % (discard.max_width as usize)) as f32 * width,
                (idx / (discard.max_width as usize)) as f32 * height,
            );

        move_tiles_writer.write(MoveTile {
            id: *tile,
            dest: new_pos,
        });

        flip_tiles_writer.write(RotateTile { id: *tile, owner });
    }
}

fn layout_wall(wall: Wall, tiles: Vec<Entity>, move_tiles_writer: &mut MessageWriter<MoveTile>) {
    let dims = wall.pos.as_vec2() * Vec2::new(TILE_WIDTH, TILE_HEIGHT);

    let top = (0..=(wall.pos.x))
        .map(|i| i as f32 * TILE_WIDTH)
        .map(|x| Vec2::new(x, dims.y) - dims / 2.0)
        .collect_vec();
    let bottom = (0..=(wall.pos.x))
        .rev()
        .map(|i| i as f32 * TILE_WIDTH)
        .map(|x| Vec2::new(x, 0.0) - dims / 2.0)
        .collect_vec();
    let right = (0..=(wall.pos.y))
        .map(|i| i as f32 * TILE_HEIGHT)
        .map(|y| Vec2::new(dims.x, y) - dims / 2.0)
        .collect_vec();
    let left = (0..=(wall.pos.y))
        .rev()
        .map(|i| i as f32 * TILE_HEIGHT)
        .map(|y| Vec2::new(0.0, y) - dims / 2.0)
        .collect_vec();

    let positions = [top, right, bottom, left].concat();

    for (i, tile_entity) in tiles.iter().enumerate() {
        let pos = positions.get(i as usize % positions.len()).unwrap();

        move_tiles_writer.write(MoveTile {
            id: *tile_entity,
            dest: *pos,
        });
    }
}

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
fn transfer_tiles(
    mut messages: MessageReader<TransferTile>,
    mut commands: Commands,
    anchors: Query<(Entity, &Anchor, Option<&Owner>)>,
    tile_collection: Query<&TileCollection>,
) {
    for &TransferTile { tile, src, dest } in messages.read() {
        if let Ok((entity, anchor, owner)) = anchors.get(dest) {
            if (matches!(anchor, Anchor::Hand(_)) && matches!(owner, Some(Owner::AI)))
                || matches!(anchor, Anchor::Wall(_))
            {
                // for entity in tile_collection.iter_descendants(entity) {
                commands
                    .entity(tile)
                    .insert(ShownFace(crate::tile::TileFace::Bottom));
                // }
            } else {
                // for entity in tile_collection.iter_descendants(entity) {
                commands
                    .entity(tile)
                    .insert(ShownFace(crate::tile::TileFace::Top));
                // }
            }
        }

        commands.entity(src).remove_related::<OwnedTile>(&[tile]);
        commands.entity(dest).add_one_related::<OwnedTile>(tile);
    }
}

fn flip_hidden_tiles(
    anchors: Query<(Entity, &Anchor, &Owner)>,
    tile_collection: Query<&TileCollection>,
    mut commands: Commands,
) {
    for (entity, anchor, owner) in &anchors {
        if !((matches!(anchor, Anchor::Hand(_)) && *owner == Owner::AI)
            || matches!(anchor, Anchor::Wall(_)))
        {
            for entity in tile_collection.iter_descendants(entity) {
                commands
                    .entity(entity)
                    .insert(ShownFace(crate::tile::TileFace::Top));
            }
        } else {
            for entity in tile_collection.iter_descendants(entity) {
                commands
                    .entity(entity)
                    .insert(ShownFace(crate::tile::TileFace::Bottom));
            }
        }
    }
}

// todo:
// - layoutdiscard function: similar to layout wall. Should check what tiles are currently in discard and all tiles and the relationship between tiles and collections
// for each discard, for each tile belonging to that discard;
//
// starts at position of the loops discard anchor, separates right by a tile width each time,
// wraps around when it reaches the edge

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
