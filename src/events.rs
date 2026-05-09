//! Events that are sent to/from the model and the view.

use bevy::prelude::*;

use crate::{level::Owner, tile::kind::TileKind};

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
    Hand(Owner),
    Wall,
}

// UI -> Model

/// We are drawing a card from somewhere
/// Note: Only draw from [`TileLocation::Wall`] or [`TileLocation::Discard`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Message)]
pub struct DrawTileMsg(TileLocation);

/// We are discarding some owner's tile from their hand
#[derive(Debug, Clone, Copy, PartialEq, Eq, Message)]
pub struct DiscardTileMsg(Owner, TileKind);

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

fn tile_transfer_msg_handler(mut messages: MessageReader<TileTransferMsg>, mut commands: Commands) {
}

fn health_update_msg_handler(mut messages: MessageReader<HealthUpdateMsg>, mut commands: Commands) {
}
