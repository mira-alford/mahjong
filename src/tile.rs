use bevy::{color::palettes::css::PURPLE, prelude::*};

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, tile_spawn_system);
    }
}

fn tile_spawn_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    spawn_tile(&mut meshes, &mut materials, &mut commands);
}

#[derive(Component)]
struct MoveCurve {
    start: Vec2,
    end: Vec2,
    t: f64,
}

fn spawn_tile(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    commands: &mut Commands,
) {
    commands.spawn((
        MoveCurve {
            start: Vec2::ZERO,
            end: Vec2::ZERO,
            t: 0.0,
        },
        Transform::default().with_scale(Vec3::splat(128.0)),
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(materials.add(Color::from(PURPLE))),
    ));
}
