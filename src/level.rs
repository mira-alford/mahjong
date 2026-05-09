use bevy::prelude::*;
use itertools::Itertools;
use std::{collections::HashSet, time::Duration};

use crate::{
    GameState,
    layout::{
        DiscardAnchor, HandAnchor, OwnedTile, Slot, TileCollection, TransferTile, UnusedAnchor,
        WallAnchor,
    },
    tile::{MoveCurve, TILE_HEIGHT, TILE_WIDTH, Tile, render::TileMaterial, spawn_tile},
};

#[derive(Resource)]
struct TransitionTimer(Timer);

/// The maximum number of tiles that a player must have in their hand
/// Usize because we can use it in iters and all that
const MAX_TILES_IN_HAND: u8 = 14;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::Game)]
#[states(scoped_entities)]
pub enum LevelState {
    #[default]
    Init,
    /// Transfers from unused or discard tiles to the wall until all are settled.
    BuildWall,
    /// Transfers from walls to each hand (alternating) on repeat until both
    /// hands are at size 14. Then waits until the tiles settle.
    Deal,
    /// In this state, the player chooses whether to draw from the wall or take from the top of the
    /// discard pile.
    Draw,
    /// In this state, the player has the opportunity to pick a tile and discard it from their hand
    Discard,
    /// Ask the player if they want to play their hand. If no, go back to drawing.
    /// On exit to this state, we switch to the other player's turn
    Play,
}

/// Resource that represents the tile owner that is 'active' (i.e., who has their turn now)
/// This is used to track whose turn it is in the lovely bevy state machine
#[derive(Resource, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct ActiveOwner(Owner);

#[derive(Component, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Owner {
    #[default]
    Player,
    AI,
}

pub fn level_plugin(app: &mut App) {
    app.init_resource::<ActiveOwner>()
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
        // deal state
        .add_systems(FixedUpdate, deal_tiles.run_if(in_state(LevelState::Deal)));
    // draw state
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
    commands.spawn((WallAnchor(IVec2::new(14, 6)), TileCollection::default()));

    // Spawn in 2 hands:
    // TODO: hand resizing system that uses window size
    commands.spawn((
        Owner::Player,
        HandAnchor(Vec2::new(-TILE_WIDTH * 8.0, TILE_HEIGHT * 4.5)),
        TileCollection::default(),
    ));
    commands.spawn((
        Owner::AI,
        HandAnchor(Vec2::new(-TILE_WIDTH * 8.0, -TILE_HEIGHT * 4.5)),
        TileCollection::default(),
    ));

    // width of the discard layout in number of tiles
    const DISCARD_LAYOUT_WIDTH: u8 = 6;
    // Spawn in 2 discards
    commands.spawn((
        Owner::Player,
        DiscardAnchor(Vec2::new(-200.0, 0.0), DISCARD_LAYOUT_WIDTH, Owner::Player),
        TileCollection::default(),
    ));
    commands.spawn((
        Owner::AI,
        DiscardAnchor(Vec2::new(200.0, 0.0), DISCARD_LAYOUT_WIDTH, Owner::AI),
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
    tile_query: Query<&Slot>,
    curves: Query<&MoveCurve>,
    mut messages: MessageWriter<TransferTile>,
    mut commands: Commands,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let mut stabilised = true;

    for sink in sinks {
        let mut set: HashSet<u8> = (0..136).into_iter().collect();
        for tile in tile_collections.iter_descendants(sink) {
            let Ok(slot) = tile_query.get(tile) else {
                continue;
            };
            set.remove(&slot.0);
        }
        if set.len() == 0 {
            set = (0..136).into_iter().collect();
        }

        let mut set = set.iter().collect_vec();

        // Are there any pieces in the unused or either discard?
        // if yes, transfer a single one (first from unused) to the wall.
        'outer: for source in sources {
            for tile_entity in tile_collections.iter_descendants(source) {
                messages.write(TransferTile {
                    tile: tile_entity,
                    src: source,
                    dest: sink,
                });
                let Some(slot) = set.pop() else {
                    continue;
                };
                commands.entity(tile_entity).insert(Slot(*slot));
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
    // mut counter: Local<usize>,
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

                if descendants.len() >= MAX_TILES_IN_HAND as usize {
                    continue;
                }

                let mut set: HashSet<u8> = (0..MAX_TILES_IN_HAND).collect();
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

fn iterate_from_turrets_to_ships(
    tiles: Query<Entity, With<OwnedTile>>,
    tile_collections: Query<Entity, With<TileCollection>>,
    attachments: Query<&OwnedTile>,
) {
    for tile in &tiles {
        for parent_collections_rel in attachments.iter_ancestors(tile) {
            let tile_collection = tile_collections.get(parent_collections_rel);

            info!(
                "tile {:?} is attached to tile collection {:?}",
                tile, tile_collection
            );
        }
    }
}

/// This is run when a tile is clicked and the state will be checked along with who owns the tile.
/// This signifies a user clicking a tile to indicate that they want to draw from the wall (as
/// opposed to taking from disard pile).
#[allow(clippy::too_many_arguments)]
pub fn tile_click_observer_draw_wall(
    event: On<Pointer<Click>>,
    state: Res<State<LevelState>>,
    // tiles
    tile_collections_entities: Query<Entity, (With<WallAnchor>, With<TileCollection>)>,
    attachments: Query<&OwnedTile>,
    tile_collections: Query<&TileCollection>,

    active_owner_res: Res<ActiveOwner>,
    owners: Query<(Entity, &Owner)>,

    mut next_state: ResMut<NextState<LevelState>>,
    mut messages: MessageWriter<TransferTile>,
) {
    let event_target = event.event_target();
    println!("clicked {:?}", event_target);

    // Can only call this on
    if !matches!(state.get(), LevelState::Draw) {
        println!("not in draw state, instead: {:?}", state.get());
        return;
    }

    let parent_collection_rels: Vec<_> = attachments.iter_ancestors(event_target).collect();
    let Some(parent_rel) = parent_collection_rels.first() else {
        warn!("couldn't find parent relationship attached to clicked tile");
        return;
    };

    let Ok(tile_owner_entity) = tile_collections_entities.get(*parent_rel) else {
        warn!(
            "couldn't find parent tile collection using parent-tile relationship, filtering by wall, so probably didn't click the wall?"
        );
        return;
    };
    info!("tile collection is: {:?}", tile_owner_entity);

    info!("running the tile draw");

    // get the entity that is currently playing
    // we do this so we can then move the tiles to the correct entity
    let active_owner = active_owner_res.0;
    let Some((active_owner_entity, _)) = owners.iter().find(|owner| *owner.1 == active_owner)
    else {
        warn!(
            "couldn't find an active entity that holds owner component: {:?}",
            active_owner
        );
        return;
    };
    info!("got the owner entity: {:?}", active_owner_entity);

    let wall_tiles: Vec<_> = tile_collections
        .iter_descendants(tile_owner_entity)
        .collect();

    let Some(drawn_tile) = wall_tiles.first() else {
        // we've run out of tiles at this point
        // so we build back our wall
        info!("going back to building the wall");
        next_state.set(LevelState::BuildWall);
        return;
    };

    info!("got the tile to draw: {:?}", drawn_tile);

    info!(
        "transfering tile {:?} from {:?} to {:?}",
        drawn_tile, tile_owner_entity, active_owner_entity
    );
    messages.write(TransferTile {
        tile: *drawn_tile,
        src: tile_owner_entity,
        dest: active_owner_entity,
    });

    next_state.set(LevelState::Discard);
}

/// This is run when a tile is clicked and the state will be checked along with who owns the tile.
/// This handles a user clicking a tile in their hand so that they can discard it
pub fn tile_click_observer_discard_from_hand(
    event: On<Pointer<Click>>,
    state: Res<State<LevelState>>,
    // tiles
    tiles: Query<Entity, With<Tile>>,
    tile_collections_entities: Query<Entity, (With<HandAnchor>, With<TileCollection>)>,
    attachments: Query<&OwnedTile>,
    tile_collections: Query<&TileCollection>,

    discard_pile_query: Query<Entity, With<DiscardAnchor>>,

    active_owner_res: Res<ActiveOwner>,
    owners: Query<(Entity, &Owner)>,

    mut next_state: ResMut<NextState<LevelState>>,
    mut messages: MessageWriter<TransferTile>,
) {
    let event_target = event.event_target();
    println!("clicked {:?}", event_target);

    // Can only call this on
    if !matches!(state.get(), LevelState::Discard) {
        println!("not in discard state, instead: {:?}", state.get());
        return;
    }

    let parent_collection_rels: Vec<_> = attachments.iter_ancestors(event_target).collect();
    let Some(parent_rel) = parent_collection_rels.first() else {
        warn!("couldn't find parent relationship attached to clicked tile");
        return;
    };

    let Ok(tile_owner_entity) = tile_collections_entities.get(*parent_rel) else {
        warn!(
            "couldn't find parent tile collection using parent-tile relationship, filtering by hand, so probably didn't click the hand?"
        );
        return;
    };
    info!("tile collection is: {:?}", tile_owner_entity);

    let discard_piles = discard_pile_query.iter().collect::<Vec<_>>();

    let discard_pile_entity = discard_piles
        .first()
        .expect("we should have a discard pile");

    info!(
        "transfering tile {:?} from {:?} to {:?}",
        event_target, tile_owner_entity, discard_pile_entity
    );
    messages.write(TransferTile {
        tile: event_target,
        src: tile_owner_entity,
        dest: *discard_pile_entity,
    });

    next_state.set(LevelState::Discard);
}
