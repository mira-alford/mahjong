use bevy::{color::palettes::css::PURPLE, ecs::event::Trigger, prelude::*};
use std::time::Instant;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, tile_spawn_system)
            .add_systems(Update, (lerp_tiles));
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
struct Tile;

#[derive(Component)]
struct MoveCurve {
    start: Vec2,
    end: Vec2,
    start_time: Instant,
    a: f32,
    b: f32,
}

fn spawn_tile(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    commands: &mut Commands,
) {
    commands
        .spawn((
            MoveCurve {
                start: Vec2::ZERO,
                end: Vec2::new(500.0, 500.0),
                start_time: Instant::now(),
                a: 1.0,
                b: 3.5,
            },
            Transform::default().with_scale(Vec3::splat(128.0)),
            Mesh2d(meshes.add(Rectangle::default())),
            MeshMaterial2d(materials.add(Color::from(PURPLE))),
        ))
        .observe(tile_click_oberver);
}

fn tile_click_oberver(event: On<Pointer<Click>>) {
    println!("clicked {:?}", event.event_target())
}

fn stretched_exp(x: f32, a: f32, b: f32) -> f32 {
    1.0 - (-(x / a).powf(b)).exp()
}

fn lerp_tiles(mut commands: Commands, mut tiles: Query<(Entity, &MoveCurve, &mut Transform)>) {
    for (entity, curve, mut transform) in &mut tiles {
        let now = Instant::now();
        let delta = now.duration_since(curve.start_time).as_secs_f32();
        let move_scalar = stretched_exp(delta, curve.a, curve.b);
        let new_pos: Vec2 = curve.start + move_scalar * (curve.end - curve.start);
        transform.translation = new_pos.extend(0.0);

        if 1.0 - move_scalar < 1e-5 {
            commands.entity(entity).remove::<MoveCurve>();
        }
    }
}
