use bevy::prelude::*;
use mahjong::tile::kind::{Dragon, Honor, TileKind};
use mahjong::tile::render::TileMaterial;
use mahjong::tile::{SharedTileData, TileBundle};

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
             shared_tile_data: Res<SharedTileData>,
             asset_server: Res<AssetServer>| {
                commands.spawn(Camera2d);
                commands.spawn(TileBundle::new(
                    &mut materials,
                    asset_server.clone(),
                    shared_tile_data.clone(),
                    TileKind::Honor(Honor::Dragon(Dragon::Red)),
                ));
            },
        )
        .run();
}
