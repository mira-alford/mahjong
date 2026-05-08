pub mod tile;
pub mod title_menu;

use bevy::prelude::*;

pub struct MahjongPlugin;

#[derive(Default, States, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    TitleMenu,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Startup, setup)
            .add_plugins((MeshPickingPlugin, tile::TilePlugin))
            .add_systems(Startup, |mut c: Commands| {
                c.spawn(Camera2d);
            })
            .add_plugins(title_menu::title_menu_plugin);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tile {
    Suit(Suit),
    Honor(Honor),
}

/// suits each with associated number between 1 and 9
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    Characters(u8),
    Circle(u8),
    Bamboo(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Honor {
    Wind(Wind),
    Dragon(Dragon),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dragon {
    Red,
    Green,
    White,
}
