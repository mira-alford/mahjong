pub mod tile;

use bevy::prelude::*;

pub struct MahjongPlugin;

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(tile::TilePlugin);
    }
}
