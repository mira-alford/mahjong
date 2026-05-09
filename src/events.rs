//! Events that are sent to/from the model and the view.

use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    layout::{Anchor, TileCollection, TransferTile},
    level::{HealthBar, LevelState, Owner},
    model::{game::GameModel, player::ActorState},
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
    pub start: TileLocation,
    pub end: TileLocation,
    pub tile: TileKind,
}

/// Model telling the view that a player's health is being updated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Message)]
pub struct HealthUpdateMsg {
    owner: Owner,
    cur_health: u32,
    max_health: u32,
}

fn draw_tile_msg_handler(
    mut messages: MessageReader<DrawTileMsg>,
    mut commands: Commands,
    mut transfer: MessageWriter<TileTransferMsg>,
    mut game_model: ResMut<GameModel>,
    actors: Query<(&mut ActorState, &Owner)>,
) {
    if messages.read().next().is_some() {
        for (mut actor, owner) in actors {
            if *owner == Owner::Player {
                let tile = game_model.wall.pop().unwrap();
                actor.draw_tile(tile);

                transfer.write(TileTransferMsg {
                    start: TileLocation::Wall,
                    end: TileLocation::Draw(Owner::Player),
                    tile,
                });
            }
        }
    }
}

fn discard_tile_msg_handler(mut messages: MessageReader<DiscardTileMsg>, mut commands: Commands) {}

fn play_tiles_msg_handler(mut messages: MessageReader<PlayTilesMsg>, mut commands: Commands) {}

fn tile_transfer_msg_handler(
    mut messages: MessageReader<TileTransferMsg>,
    mut commands: Commands,
    anchor_query: Query<(Entity, &Anchor, Option<&Owner>)>,
    tile_collections: Query<&TileCollection>,
    tiles: Query<&Tile>,
    mut transfer_writer: MessageWriter<TransferTile>,
) {
    let mut seen = HashSet::new();
    for message in messages.read() {
        fn find_anchor(
            anchor_query: Query<'_, '_, (Entity, &Anchor, Option<&Owner>)>,
            tile_location: &TileLocation,
        ) -> Option<Entity> {
            anchor_query
                .iter()
                .find(|(_, anchor, owner)| match (**anchor, *owner) {
                    (Anchor::Hand(_), Some(owner)) => {
                        matches!(tile_location, TileLocation::Hand(o, _) if *o == *owner)
                    }
                    (Anchor::Wall(_), _) => matches!(tile_location, TileLocation::Wall),
                    (Anchor::Discard(_), Some(owner)) => {
                        matches!(tile_location, TileLocation::Discard(o) if *o == *owner)
                    }
                    (Anchor::Draw(_), Some(owner)) => {
                        matches!(tile_location, TileLocation::Draw(o) if *o == *owner)
                    }
                    (Anchor::Unused(_), _) => false,
                    _ => false,
                })
                .map(|t| t.0)
        }

        let (Some(start_anchor), Some(end_anchor)) = (
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
                    && !seen.contains(&entity)
            })
        else {
            warn!(tile=?message.tile, "Unable to tile.");
            continue;
        };

        seen.insert(tile);

        transfer_writer.write(TransferTile {
            tile,
            src: start_anchor,
            dest: end_anchor,
        });
    }
}

fn health_update_msg_handler(
    mut messages: MessageReader<HealthUpdateMsg>,
    mut healthbars: Query<(&mut Text, &Owner), With<HealthBar>>,
) {
    for msg in messages.read() {
        // get the text component that matches the message
        let Some((mut found_text, owner)) = healthbars
            .iter_mut()
            .find(|(_, healthbar_owner)| msg.owner == **healthbar_owner)
        else {
            warn!(msg = ?msg, "got a message to update healthbar, but couldn't find the matching text for the owner");
            continue;
        };

        let owner_name: String = match owner {
            Owner::Player => "Your".into(),
            Owner::AI => "Enemy".into(),
        };

        let new_text = format!(
            "{} Health: {}/{}",
            owner_name, msg.cur_health, msg.max_health
        );

        **found_text = new_text;
    }
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

    let msg = DrawTileMsg(draw_location);
    info!(msg=?msg, "sending message");
    messages.write(msg);

    info!(state=?LevelState::Discard, "transitioning state");
    next_state.set(LevelState::Discard);
}
pub fn discard_tile(
    anchor: Anchor,
    owner: Option<Owner>,
    tile: Tile,
    index: usize,
    state: Res<GameModel>,
    mut messages: MessageWriter<DiscardTileMsg>,
    mut next_state: ResMut<NextState<LevelState>>,
) {
    dbg!(&index);
    let discard_location: TileLocation = match (anchor, owner) {
        (Anchor::Hand(_), Some(hand_owner)) if state.turn == hand_owner => {
            TileLocation::Hand(hand_owner, index) // TODO: Don't worry about it
        }
        (Anchor::Draw(_), Some(draw_owner)) if state.turn == draw_owner => {
            TileLocation::Draw(draw_owner)
        }
        _ => {
            debug!(anchor=?anchor, owner=?owner,"not handling case in discard tile");
            return;
        }
    };

    let msg = DiscardTileMsg(discard_location, tile.kind);
    info!(msg=?msg, "sending message");
    messages.write(msg);

    info!(state=?LevelState::Play, "transitioning state");
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

    let msg = PlayTilesMsg(owner);
    info!(msg=?msg, "sending message");
    messages.write(msg);

    // we swap the player's turn
    state.turn = match &state.turn {
        Owner::Player => Owner::AI,
        Owner::AI => Owner::Player,
    };

    // go back to the draw state
    info!(state=?LevelState::Draw, "transitioning state");
    next_state.set(LevelState::Draw);
}
