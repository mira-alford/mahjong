use bevy::{color::palettes::css::PURPLE, prelude::*};
use std::time::{Duration, Instant};

use crate::{
    GameState,
    layout::{DiscardAnchor, HandAnchor, OwnedTile, TileCollection, UnusedAnchor, WallAnchor},
    tile::{render::TileMaterial, spawn_tile},
};

#[derive(Resource)]
struct TransitionTimer(Timer);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::Game)]
#[states(scoped_entities)]
enum LevelState {
    #[default]
    Init,
    /// Transfers from unused or discard tiles to the wall until all are settled.
    BuildWall,
    /// Transfers from walls to each hand (alternating) on repeat until both
    /// hands are at size 14. Then waits until the tiles settle.
    Deal,
    /// On entry to this state, toggle whose turn it is to the other.
    /// Transfer from the wall to a players "draw" location once.
    Draw,
    /// Enable systems to allow the player to select a tile in their hand
    /// and swap it. Once selected, go to the steal stage.
    Swap,
    /// Transition the tile into the secret "steal" pile. At any point
    /// the other opponent can either steal or ignore the piece.
    /// A steal event includes the piece they are replacing.
    /// If no steal, transition to discard. If steal, transition into their hand.
    Steal,
    /// Ask the player if they want to play their hand. If no, go back to drawing.
    Play,
}

#[derive(Resource, Default)]
enum Turn {
    #[default]
    Player,
    AI,
}

#[derive(Component, Default)]
enum Owner {
    #[default]
    Player,
    AI,
}

pub fn level_plugin(app: &mut App) {
    app.init_resource::<Turn>()
        .insert_resource(TransitionTimer(Timer::new(
            Duration::from_millis(100),
            TimerMode::Repeating,
        )))
        .add_sub_state::<LevelState>()
        .add_systems(OnEnter(LevelState::Init), init_level)
        .add_systems(
            FixedUpdate,
            build_wall.run_if(in_state(LevelState::BuildWall)),
        )
        .add_systems(FixedUpdate, deal_tiles.run_if(in_state(LevelState::Deal)));
}

/// Initializes the level.
/// There will presumably be some kind of inter level data saying what
/// the players "deck" is that initially populates the unused tiles collection.
/// For now its hardcoded.
fn init_level(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TileMaterial>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<LevelState>>,
    asset_server: Res<AssetServer>,
) {
    let tile_face_material = materials.add(TileMaterial::new(
        asset_server.load("front-face-placeholder.png"),
    ));
    let tile_mesh = meshes.add(Rectangle::default());

    // Spawn in the unused pile.
    let unused_id = commands
        .spawn((UnusedAnchor(Vec2::ZERO), TileCollection::default()))
        .id();

    // Just hard spawning 32 unused tiles for now :)
    // TODO: Eventually replace this
    for _ in 0..32 {
        let tile_id = spawn_tile(&mut commands, &tile_face_material, &tile_mesh);
        commands.entity(tile_id).insert(OwnedTile(unused_id));
    }

    // Spawn in the wall!
    // TODO: wall resizing system that uses window size
    commands.spawn((WallAnchor(Vec2::ONE * 800.0), TileCollection::default()));

    // Spawn in 2 hands:
    // TODO: hand resizing system that uses window size
    commands.spawn((
        Owner::Player,
        HandAnchor(Vec2::new(-500.0, -300.0), 1000.0),
        TileCollection::default(),
    ));
    commands.spawn((
        Owner::AI,
        HandAnchor(Vec2::new(-500.0, 300.0), 1000.0),
        TileCollection::default(),
    ));

    // Spawn in 2 discards
    commands.spawn((
        Owner::Player,
        DiscardAnchor(Vec2::new(-200.0, 0.0)),
        TileCollection::default(),
    ));
    commands.spawn((
        Owner::AI,
        DiscardAnchor(Vec2::new(200.0, 0.0)),
        TileCollection::default(),
    ));

    // Go to wall building
    next_state.set(LevelState::BuildWall);
}

/// Builds the wall, drawing tiles from the unused collection + both discards until they are all empty.
fn build_wall(
    mut timer: ResMut<TransitionTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<LevelState>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let mut stabilised = true;

    // Are there any pieces in the unused or either discard?
    // if yes, transfer a single one (first from unused) to the wall.
    let pieces_remaining = true;
    if pieces_remaining {
        stabilised = false;
    }

    let pieces_stabilised = false;
    if !pieces_stabilised {
        stabilised = false;
    }

    // No more pieces? then transition state to dealing.
    if stabilised {
        next_state.set(LevelState::Deal);
    }
}

/// Repeatedly plays a single tile into any hand with size < 14.
fn deal_tiles(
    mut timer: ResMut<TransitionTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<LevelState>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    // If both players hands are at least size 14, we go to the draw state.
    let full = true;
    if full {
        next_state.set(LevelState::Draw);
        return;
    }

    // If the wall is empty, go back to building the wall
    let wall_empty = false;
    if wall_empty {
        next_state.set(LevelState::BuildWall);
        return;
    }

    // Otherwise, we maintain this state.
    // Play a tile into any hand of size < 14.
    // ...
}
