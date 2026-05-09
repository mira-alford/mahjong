use crate::tile::kind::TileKind;
use bevy::prelude::*;

/// The global state of the game
#[derive(Resource, Default)]
pub struct GameModel {
    /// The deck that both players draw from
    pub wall: Vec<TileKind>,
}
