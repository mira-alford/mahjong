use bevy::prelude::*;
use mahjong::tile::render::TileMaterial;
use mahjong::tile::spawn_tile;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "mahjong".into(),
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            MeshPickingPlugin,
            mahjong::tile::TilePlugin,
        ))
        .add_systems(
            Startup,
            |mut meshes: ResMut<Assets<Mesh>>,
             mut materials: ResMut<Assets<TileMaterial>>,
             mut commands: Commands,
             asset_server: Res<AssetServer>| {
                commands.spawn(Camera2d);
                spawn_tile(
                    &mut commands,
                    &meshes.add(Rectangle::from_size(Vec2::new(1.0, 4.0 / 3.0))),
                    &mut materials,
                    asset_server.clone(),
                );
            },
        )
        .run();
}
