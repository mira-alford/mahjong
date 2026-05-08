use bevy::prelude::*;

mod title_menu;

#[derive(Default, States, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    TitleMenu,
}
