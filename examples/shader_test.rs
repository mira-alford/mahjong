use bevy::picking::hover::Hovered;
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
            |mut materials: ResMut<Assets<TileMaterial>>,
             mut commands: Commands,
             shared_tile_data: Res<SharedTileData>,
             asset_server: Res<AssetServer>| {
                commands.spawn(Camera2d);
                commands
                    .spawn(TileBundle::new(
                        &mut materials,
                        asset_server.clone(),
                        shared_tile_data.clone(),
                        TileKind::Honor(Honor::Dragon(Dragon::Red)),
                    ))
                    .observe(hover_system)
                    .observe(unhover_system);
            },
        )
        .run();
}

fn hover_system(
    event: On<Pointer<Over>>,
    time: Res<Time>,
    mut materials: ResMut<Assets<TileMaterial>>,
    mut material_query: Query<&mut MeshMaterial2d<TileMaterial>>,
    mut t: Local<f32>,
) {
    let material_handle = material_query.get_mut(event.entity).unwrap();

    let material = materials.get_mut(&material_handle.0).unwrap();
    let curr_scale = material.clone().get_scale();

    *t = (1.0 - *t) * 0.1 * time.delta_secs();

    material.set_scale(&curr_scale.lerp(Vec2::new(2.0, 2.0), *t));

    material.set_tint(Color::linear_rgba(1.0, 0.5, 0.5, 1.0));
}

fn unhover_system(
    event: On<Pointer<Out>>,
    time: Res<Time>,
    mut materials: ResMut<Assets<TileMaterial>>,
    mut material_query: Query<&mut MeshMaterial2d<TileMaterial>>,
    mut t: Local<f32>,
) {
    let material_handle = material_query.get_mut(event.entity).unwrap();

    let material = materials.get_mut(&material_handle.0).unwrap();
    let curr_scale = material.clone().get_scale();

    *t = (1.0 - *t) * 0.1 * time.delta_secs();

    material.set_scale(&curr_scale.lerp(Vec2::new(1.0, 1.0), *t));

    material.set_tint(Color::linear_rgba(1.0, 1.0, 1.0, 1.0));
}
