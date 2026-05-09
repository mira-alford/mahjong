//! Events that are sent to/from the model and the view.

use bevy::prelude::*;

use crate::{
    layout::{Anchor, TileCollection, TransferTile},
    level::{LevelState, Owner},
    model::game::GameModel,
    tile::{Tile, kind::TileKind},
};

pub fn event_plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            draw_tile_msg_handler,
            discard_tile_msg_handler,
            play_tiles_msg_handler,
            tile_transfer_msg_handler,
            health_update_msg_handler,
        ),
    )
    // ui -> model
    .add_message::<DrawTileMsg>()
    .add_message::<DiscardTileMsg>()
    .add_message::<PlayTilesMsg>()
    // model -> ui
    .add_message::<TileTransferMsg>()
    .add_message::<HealthUpdateMsg>();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileLocation {
    Discard(Owner),
    Hand(Owner, usize),
    Draw(Owner),
    Wall,
}

// UI -> Model

/// We are drawing a card from somewhere
/// Note: Only draw from [`TileLocation::Wall`] or [`TileLocation::Discard`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Message)]
pub struct DrawTileMsg(TileLocation);

/// We are discarding some owner's tile from their hand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Message)]
pub struct DiscardTileMsg(TileLocation, TileKind);

/// We are playing the hand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Message)]
pub struct PlayTilesMsg(Owner);

// Model -> UI

/// Model telling the view that a tile is being transferred
#[derive(Debug, Clone, Copy, PartialEq, Eq, Message)]
pub struct TileTransferMsg {
    start: TileLocation,
    end: TileLocation,
    tile: TileKind,
}

/// Model telling the view that a player's health is being updated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Message)]
pub struct HealthUpdateMsg {
    owner: Owner,
    health: u32,
}

fn draw_tile_msg_handler(mut messages: MessageReader<DrawTileMsg>, mut commands: Commands) {}

fn discard_tile_msg_handler(mut messages: MessageReader<DiscardTileMsg>, mut commands: Commands) {}

fn play_tiles_msg_handler(mut messages: MessageReader<PlayTilesMsg>, mut commands: Commands) {}

fn tile_transfer_msg_handler(
    mut messages: MessageReader<TileTransferMsg>,
    mut commands: Commands,
    anchor_query: Query<(Entity, &Anchor, &Owner)>,
    tile_collections: Query<&TileCollection>,
    tiles: Query<&Tile>,
    mut transfer_writer: MessageWriter<TransferTile>,
) {
    for message in messages.read() {
        // fn fun_name(
        //     anchor_query: Query<'_, '_, (Entity, &Anchor, &Owner)>,
        //     message: &TileTransferMsg,
        // ) -> Option<(Entity, &Anchor, &Owner)> {
        fn find_anchor(
            anchor_query: Query<'_, '_, (Entity, &Anchor, &Owner)>,
            tile_location: &TileLocation,
        ) -> Option<Entity> {
            anchor_query
                .iter()
                .find(|(_, anchor, owner)| match anchor {
                    Anchor::Hand(_) => {
                        matches!(tile_location, TileLocation::Hand(o, _) if o == *owner)
                    }
                    Anchor::Wall(_) => matches!(tile_location, TileLocation::Wall),
                    Anchor::Discard(_) => {
                        matches!(tile_location, TileLocation::Discard(o) if o == *owner)
                    }
                    Anchor::Unused(_) => false,
                    Anchor::Draw(_) => {
                        matches!(tile_location, TileLocation::Draw(o) if o == *owner)
                    }
                })
                .map(|t| t.0)
        }

        let (Some((start_anchor)), Some((end_anchor))) = (
            find_anchor(anchor_query, &message.start),
            find_anchor(anchor_query, &message.end),
        ) else {
            warn!(start=?message.start, end=?message.end, "Unable to find start and end anchor.");
            continue;
        };

        let Some(tile) = tile_collections
            .iter_descendants(start_anchor)
            .find(|&entity| {
                tiles
                    .get(entity)
                    .is_ok_and(|tile| tile.kind == message.tile)
            })
        else {
            warn!(tile=?message.tile, "Unable to tile.");
            continue;
        };

        transfer_writer.write(TransferTile {
            tile,
            src: start_anchor,
            dest: end_anchor,
        });
    }
}

fn health_update_msg_handler(mut messages: MessageReader<HealthUpdateMsg>, mut commands: Commands) {
}

pub fn draw_tile(
    anchor: Anchor,
    owner: Option<Owner>,
    state: Res<GameModel>,
    mut messages: MessageWriter<DrawTileMsg>,
    mut next_state: ResMut<NextState<LevelState>>,
) {
    let draw_location: TileLocation = match (anchor, owner) {
        (Anchor::Wall(_), None) => TileLocation::Wall,
        (Anchor::Discard(_), Some(discard_owner)) if state.turn != discard_owner => {
            TileLocation::Discard(discard_owner)
        }
        _ => {
            debug!(anchor=?anchor, owner=?owner,"not handling case in draw tile");
            return;
        }
    };

    messages.write(DrawTileMsg(draw_location));
    next_state.set(LevelState::Discard);
}
pub fn discard_tile(
    anchor: Anchor,
    owner: Option<Owner>,
    tile: Tile,
    state: Res<GameModel>,
    mut messages: MessageWriter<DiscardTileMsg>,
    mut next_state: ResMut<NextState<LevelState>>,
) {
    let discard_location: TileLocation = match (anchor, owner) {
        (Anchor::Hand(_), Some(hand_owner)) if state.turn == hand_owner => {
            TileLocation::Hand(hand_owner, 0) // TODO: Don't worry about it
        }
        (Anchor::Draw(_), Some(draw_owner)) if state.turn == draw_owner => {
            TileLocation::Draw(draw_owner)
        }
        _ => {
            debug!(anchor=?anchor, owner=?owner,"not handling case in discard tile");
            return;
        }
    };

    messages.write(DiscardTileMsg(discard_location, tile.kind));
    next_state.set(LevelState::Play);
}
pub fn play_tile(
    anchor: Anchor,
    owner_opt: Option<Owner>,
    mut state: ResMut<GameModel>,
    mut messages: MessageWriter<PlayTilesMsg>,
    mut next_state: ResMut<NextState<LevelState>>,
) {
    // we can only play tiles when the tile we clicked belongs to the hand
    // also, we can only play tiles when we click **our hand**
    let owner: Owner = match (owner_opt, anchor) {
        (Some(cur_owner), Anchor::Hand(_)) if cur_owner == state.turn => cur_owner,
        _ => {
            info!(anchor=?anchor, owner=?owner_opt,"not handling case in play tile");
            return;
        }
    };

    messages.write(PlayTilesMsg(owner));

    // we swap the player's turn
    state.turn = match &state.turn {
        Owner::Player => Owner::AI,
        Owner::AI => Owner::Player,
    };

    // go back to the draw state
    next_state.set(LevelState::Draw);
}
