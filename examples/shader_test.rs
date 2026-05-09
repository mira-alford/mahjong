use bevy::prelude::*;
use mahjong::tile::kind::{Dragon, Honor, TileKind};
use mahjong::tile::render::{HoveringAnimation, TileMaterial};
use mahjong::tile::{SharedTileData, ShownFace, TileBundle};

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
            PostStartup,
            |mut materials: ResMut<Assets<TileMaterial>>,
             mut commands: Commands,
             shared_tile_data: Res<SharedTileData>,
             asset_server: Res<AssetServer>| {
                commands.spawn(Camera2d);
                let mut tile = commands.spawn(TileBundle::new(
                    &mut materials,
                    asset_server.clone(),
                    shared_tile_data.clone(),
                    TileKind::Honor(Honor::Dragon(Dragon::Red)),
                ));

                // insert this on it to make it react to being hovered
                tile.insert(HoveringAnimation).observe(flip_on_click);

                let mut tile = commands.spawn(TileBundle::new(
                    &mut materials,
                    asset_server.clone(),
                    shared_tile_data.clone(),
                    TileKind::Honor(Honor::Dragon(Dragon::Red)),
                ));

                tile.insert((
                    HoveringAnimation,
                    Transform::default().with_translation(Vec3::new(120.0, 0.0, 0.0)),
                ))
                .observe(flip_on_click);
            },
        )
        .run();
}

fn flip_on_click(event: On<Pointer<Click>>, mut query: Query<&mut ShownFace>) {
    let mut shown_face = query.get_mut(event.event_target()).expect("ahahahahahah");

    match shown_face.0 {
        mahjong::tile::TileFace::Top => shown_face.0 = mahjong::tile::TileFace::Bottom,
        mahjong::tile::TileFace::Bottom => shown_face.0 = mahjong::tile::TileFace::Top,
    }
}
