#![allow(clippy::type_complexity)]

pub mod layout;
pub mod level;
pub mod tile;
pub mod title_menu;

use bevy::prelude::*;

pub struct MahjongPlugin;

#[derive(Default, States, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    TitleMenu,
    Game,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Startup, setup)
            .add_plugins((MeshPickingPlugin, tile::TilePlugin))
            .add_plugins(title_menu::title_menu_plugin)
            .add_plugins(level::level_plugin)
            .add_plugins(layout::layout_plugin);
    }
}
