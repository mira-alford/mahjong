use bevy::prelude::*;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, tile_spawn_system);
    }
}

fn tile_spawn_system(mut commands: Commands) {
    spawn_tile(&mut commands);
}

#[derive(Component)]
struct MoveCurve {
    start: Vec2,
    end: Vec2,
    t: f64,
}

fn spawn_tile(commands: &mut Commands) {
    commands.spawn((
        MoveCurve {
            start: Vec2::ZERO,
            end: Vec2::ZERO,
            t: 0.0,
        },
        Transform::default(),
    ));
}
