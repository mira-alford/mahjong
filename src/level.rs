use bevy::prelude::*;
use std::{collections::HashSet, time::Duration};

use crate::{
    GameState,
    layout::{
        DiscardAnchor, HandAnchor, OwnedTile, Slot, TileCollection, TransferTile, UnusedAnchor,
        WallAnchor,
    },
    tile::{MoveCurve, TILE_HEIGHT, TILE_WIDTH, render::TileMaterial, spawn_tile},
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
pub enum Owner {
    #[default]
    Player,
    AI,
}

pub fn level_plugin(app: &mut App) {
    app.init_resource::<Turn>()
        .insert_resource(TransitionTimer(Timer::new(
            Duration::from_millis(1),
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
    let tile_mesh = meshes.add(Rectangle::from_size(Vec2::new(TILE_WIDTH, TILE_HEIGHT)));

    // Spawn in the unused pile.
    let unused_id = commands
        .spawn((UnusedAnchor(Vec2::ZERO), TileCollection::default()))
        .id();

    // Just hard spawning 32 unused tiles for now :)
    // TODO: Eventually replace this
    for _ in 0..136 {
        let tile_id = spawn_tile(
            &mut commands,
            &tile_mesh,
            &mut materials,
            asset_server.clone(),
        );
        commands.entity(tile_id).insert(OwnedTile(unused_id));
    }

    // Spawn in the wall!
    // TODO: wall resizing system that uses window size
    commands.spawn((
        WallAnchor(Vec2::new(1000.0, 1200.0), IVec2::ONE * 13),
        TileCollection::default(),
    ));

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
    sources: Query<Entity, Or<(With<UnusedAnchor>, With<DiscardAnchor>)>>,
    sinks: Query<Entity, With<WallAnchor>>,
    tile_collections: Query<&TileCollection>,
    curves: Query<&MoveCurve>,
    mut messages: MessageWriter<TransferTile>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let mut stabilised = true;

    for sink in sinks {
        // Are there any pieces in the unused or either discard?
        // if yes, transfer a single one (first from unused) to the wall.
        'outer: for source in sources {
            for tile_entity in tile_collections.iter_descendants(source) {
                messages.write(TransferTile {
                    tile: tile_entity,
                    src: source,
                    dest: sink,
                });
                stabilised = false;
                break 'outer;
            }
        }
    }

    let len = curves.iter().len();
    // No more pieces? then transition state to dealing.
    if len == 0 && stabilised {
        next_state.set(LevelState::Deal);
    }
}

/// Repeatedly plays a single tile into any hand with size < 14.
fn deal_tiles(
    mut timer: ResMut<TransitionTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<LevelState>>,
    sources: Query<Entity, With<WallAnchor>>,
    sinks: Query<Entity, With<HandAnchor>>,
    tile_collections: Query<&TileCollection>,
    tile_query: Query<(Entity, Option<&Slot>)>,
    mut messages: MessageWriter<TransferTile>,
    mut commands: Commands,
    mut counter: Local<usize>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    // Choose a sink;
    // let sinks: Vec<_> = sinks.iter().collect();
    // *counter += 1;
    // *counter %= sinks.len();
    // let sink = sinks[*counter];

    for source in sources {
        for tile_entity in tile_collections.iter_descendants(source) {
            for sink in sinks {
                // the sink is a hand, and the descendants of sink are Tiles
                let descendants: Vec<_> = tile_collections.iter_descendants(sink).collect();

                if descendants.len() >= 14 {
                    continue;
                }

                let mut set: HashSet<u8> = (0..14).into_iter().collect();
                for descendant in descendants {
                    // if the descendant (Tile in a hand?) already has a slot, add that slot number
                    // to a set of currently filled slots.
                    //
                    // Then once this loop is done, iterate over the set and allocate all of the
                    // missing slots between 0 and 13 to a descendant (tile in hand)
                    if let Ok((_, Some(Slot(x)))) = tile_query.get(descendant) {
                        set.remove(x);
                    }
                }

                commands
                    .entity(tile_entity)
                    .insert(Slot(set.into_iter().next().unwrap()));

                messages.write(TransferTile {
                    tile: tile_entity,
                    src: source,
                    dest: sink,
                });
                return;
            }
            next_state.set(LevelState::Draw);
            return;
        }
    }
    next_state.set(LevelState::BuildWall);

    // If both players hands are at least size 14, we go to the draw state.

    // Otherwise, we maintain this state.
    // Play a tile into any hand of size < 14.
    // ...
}
