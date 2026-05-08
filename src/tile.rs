use bevy::{color::palettes::css::PURPLE, prelude::*};
use std::time::Instant;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (lerp_tiles, update_tile_materials));
    }
}

fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut commands: Commands,
) {
    // TODO: make this editiable on the plugin
    commands.insert_resource(TileBackMaterial(materials.add(Color::WHITE)));

    let tile_face_material = materials.add(ColorMaterial::from_color(PURPLE));
    let tile_mesh = meshes.add(Rectangle::default());

    spawn_tile(&mut commands, &tile_face_material, &tile_mesh);
}

/// marker componenet for tiles
#[derive(Component)]
struct Tile {}

/// material for a tiles face
#[derive(Component)]
struct TileFaceMaterial(Handle<ColorMaterial>);

// Shared resource for the back of tiles (since all tiles are same on back)
#[derive(Resource)]
struct TileBackMaterial(Handle<ColorMaterial>);

/// the currently up facing face of a tile, i.e. the face you can see
#[derive(Component, Default)]
struct ShownFace(TileFace);

#[derive(Default)]
enum TileFace {
    #[default]
    Top,
    Bottom,
}

/// movement curve
#[derive(Component)]
struct MoveCurve {
    start: Vec2,
    end: Vec2,
    start_time: Instant,
    a: f32,
    b: f32,
}

/// spawns a tile with a specified front facing material, and a mesh
fn spawn_tile(commands: &mut Commands, material: &Handle<ColorMaterial>, mesh: &Handle<Mesh>) {
    commands
        .spawn((
            Tile {},
            TileFaceMaterial(material.clone()),
            ShownFace::default(),
            MoveCurve {
                start: Vec2::ZERO,
                end: Vec2::new(500.0, 500.0),
                start_time: Instant::now(),
                a: 1.0,
                b: 3.5,
            },
            Transform::default().with_scale(Vec3::splat(128.0)),
            // TODO: this should probably be done with a resource specified when plugin made
            Mesh2d(mesh.clone()),
            MeshMaterial2d(material.clone()),
        ))
        .observe(tile_click_oberver);
}

fn update_tile_materials(
    mut query: Query<
        (
            &ShownFace,
            &TileFaceMaterial,
            &mut MeshMaterial2d<ColorMaterial>,
        ),
        Changed<ShownFace>,
    >,
    bottom_material: Res<TileBackMaterial>,
) {
    for (face, top_material, mut material) in query.iter_mut() {
        match face.0 {
            TileFace::Top => material.0 = top_material.0.clone(),
            TileFace::Bottom => material.0 = bottom_material.0.clone(),
        }
    }
}

fn tile_click_oberver(event: On<Pointer<Click>>, mut query: Query<&mut ShownFace>) {
    let event_target = event.event_target();

    println!("clicked {:?}", event_target);

    let mut face = query
        .get_mut(event_target)
        .expect("expected clicked tile to have ShownFace componenet");

    match face.0 {
        TileFace::Top => face.0 = TileFace::Bottom,
        TileFace::Bottom => face.0 = TileFace::Top,
    }
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
