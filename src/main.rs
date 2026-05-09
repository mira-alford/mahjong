use bevy::{prelude::*, window::WindowResolution};
use mahjong::MahjongPlugin;

fn main() {
    App::new()
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
            MahjongPlugin,
        ))
        .run();
}
