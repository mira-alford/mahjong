#![allow(clippy::type_complexity)]

pub mod events;
pub mod layout;
pub mod level;
pub mod model;
pub mod player_select;
pub mod tile;
pub mod title_menu;
pub mod tutorial;

use bevy::{prelude::*, window::WindowResolution};

pub struct MahjongPlugin;

#[derive(Default, States, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    #[default]
    TitleMenu,
    PlayerSelect,
    Game,
    Tutorial,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}

impl Plugin for MahjongPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_plugins((
                DefaultPlugins.set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "mahjong".into(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        resolution: WindowResolution::new(1280, 720),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                MeshPickingPlugin,
                tile::TilePlugin,
                title_menu::title_menu_plugin,
                player_select::player_select_plugin,
                level::level_plugin,
                events::event_plugin,
                layout::layout_plugin,
                tutorial::tutorial_plugin,
            ))
            .init_state::<GameState>();
    }
}
