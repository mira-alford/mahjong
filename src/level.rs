use bevy::prelude::*;
use itertools::Itertools;
use rand::seq::SliceRandom;
use std::{collections::HashSet, fmt, time::Duration};

use crate::{
    GameState,
    events::{TileLocation, TileTransferMsg},
    layout::{Anchor, Discard, Hand, OwnedTile, Slot, TileCollection, TransferTile, Unused, Wall},
    model::{
        game::GameModel,
        player::{ActorState, PlayerLoadout},
    },
    tile::{
        MoveCurve, SharedTileData, ShownFace, TILE_HEIGHT, TILE_WIDTH, TileBundle, kind::TileKind,
        render::TileMaterial, tile_click_oberver,
    },
};

#[derive(Resource)]
struct TransitionTimer(Timer);

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
    Draw,
    Discard,
    Play,
}

/// Marker component that signifies that a thing is a health indicator
/// so that we can search for them to update them
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct HealthBar;

#[derive(Resource, Default, Clone, Copy, Debug)]
enum Turn {
    #[default]
    Player,
    AI,
}

#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    #[default]
    Player,
    AI,
}

pub fn level_plugin(app: &mut App) {
    app.init_resource::<GameModel>()
        .init_resource::<Turn>()
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
        .add_systems(OnEnter(LevelState::Deal), deal_tiles);
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
    mut game_model: ResMut<GameModel>,
    player_loadout: Res<PlayerLoadout>,
    asset_server: Res<AssetServer>,
    shared_tile_data: Res<SharedTileData>,
    assets: Res<AssetServer>,
    mut transfer_tile: MessageWriter<TileTransferMsg>,
) {
    commands.spawn((
        AudioPlayer::new(assets.load("audio/mahjong to the death.ogg")),
        PlaybackSettings::LOOP,
    ));
    // Init the model to a good state
    let mut deck = player_loadout.full_deck.clone();
    let mut rng = rand::rng();
    deck.shuffle(&mut rng);

    game_model.wall = deck;

    // Spawn the player and enemy state
    commands.spawn((Owner::Player, player_loadout.actor_state(vec![])));
    commands.spawn((Owner::AI, ActorState::default_enemy(vec![])));

    let tile_mesh = meshes.add(Rectangle::from_size(Vec2::new(TILE_WIDTH, TILE_HEIGHT)));

    // Spawn in the unused pile.
    let unused_id = commands
        .spawn((Anchor::Unused(Unused), TileCollection::default()))
        .id();

    // Just hard spawning 32 unused tiles for now :)
    // TODO: Eventually replace this
    for tile in player_loadout.full_deck.clone() {
        let tile_id = commands
            .spawn(
                TileBundle::new(
                    &mut materials,
                    asset_server.clone(),
                    shared_tile_data.clone(),
                    tile,
                )
                .shown_face(crate::tile::TileFace::Bottom),
            )
            .observe(tile_click_oberver)
            .id();
        commands.entity(tile_id).insert(OwnedTile(unused_id));
    }

    // Spawn in the wall!
    // TODO: wall resizing system that uses window size
    commands.spawn((
        Anchor::Wall(Wall {
            pos: IVec2::new(14, 6),
        }),
        TileCollection::default(),
    ));

    commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Px(50.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },))
        .with_children(|builder| {
            // ai health indicator
            builder.spawn((
                Text::new(""),
                Owner::AI,
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                HealthBar,
                Node {
                    margin: UiRect::top(Val::Px(-50.0)),
                    ..default()
                },
            ));

            // player health indicator
            builder.spawn((
                Text::new(""),
                Owner::Player,
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                HealthBar,
                Node {
                    margin: UiRect::top(Val::Px(50.0)),
                    ..default()
                },
            ));
        });

    // Spawn in 2 hands:
    // TODO: hand resizing system that uses window size
    commands.spawn((
        Owner::Player,
        Anchor::Hand(Hand {
            pos: Vec2::new(-TILE_WIDTH * 8.0, -TILE_HEIGHT * 4.5),
        }),
        TileCollection::default(),
    ));
    commands.spawn((
        Owner::AI,
        Anchor::Hand(Hand {
            pos: Vec2::new(TILE_WIDTH * 8.0, TILE_HEIGHT * 4.5),
        }),
        TileCollection::default(),
    ));

    commands.spawn((
        Owner::Player,
        Anchor::Draw(crate::layout::Draw(Vec2::new(
            TILE_WIDTH * 10.0,
            -TILE_HEIGHT * 4.5,
        ))),
        TileCollection::default(),
    ));
    commands.spawn((
        Owner::AI,
        Anchor::Draw(crate::layout::Draw(Vec2::new(
            -TILE_WIDTH * 10.0,
            TILE_HEIGHT * 4.5,
        ))),
        TileCollection::default(),
    ));

    // width of the discard layout in number of tiles
    const DISCARD_LAYOUT_WIDTH: u8 = 6;
    // Spawn in 2 discards
    commands.spawn((
        Owner::Player,
        Anchor::Discard(Discard {
            pos: Vec2::new(-200.0, 0.0),
            max_width: DISCARD_LAYOUT_WIDTH,
        }),
        TileCollection::default(),
    ));
    commands.spawn((
        Owner::AI,
        Anchor::Discard(Discard {
            pos: Vec2::new(200.0, 0.0),
            max_width: DISCARD_LAYOUT_WIDTH,
        }),
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
    sources: Query<(Entity, &Anchor)>,
    sinks: Query<(Entity, &Anchor)>,
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

    for (sink_entity, sink_anchor) in sinks {
        if !matches!(sink_anchor, Anchor::Wall(_)) {
            continue;
        }
        let mut set: HashSet<u8> = (0..136).into_iter().collect();
        for tile in tile_collections.iter_descendants(sink_entity) {
            let Ok(slot) = tile_query.get(tile) else {
                continue;
            };
            set.remove(&slot.0);
        }
        if set.len() == 0 {
            set = (0..136).into_iter().collect();
        }

        let mut set = set.iter().collect_vec();

        // sources: Query<Entity, <(With<UnusedAnchor>, With<DiscardAnchor>)>>,

        // Are there any pieces in the unused or either discard?
        // if yes, transfer a single one (first from unused) to the wall.
        'outer: for (source_entity, source_anchor) in sources {
            if !matches!(source_anchor, Anchor::Unused(_) | Anchor::Discard(_)) {
                continue;
            }

            for tile_entity in tile_collections.iter_descendants(source_entity) {
                messages.write(TransferTile {
                    tile: tile_entity,
                    src: source_entity,
                    dest: sink_entity,
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
    sources: Query<(Entity, &Anchor)>,
    sinks: Query<(Entity, &Anchor)>,
    tile_collections: Query<&TileCollection>,
    tile_query: Query<&Slot>,
    mut messages: MessageWriter<TileTransferMsg>,
    mut commands: Commands,
    mut counter: Local<usize>,
    mut model: ResMut<GameModel>,
    mut oneshot: Local<bool>,
    mut actors: Query<(&mut ActorState, &Owner)>,
) {
    /*
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    // sources: Query<(Entity, &WallAnchor)>,
    // sinks: Query<Entity, With<HandAnchor>>,

    // Choose a sink;
    // let sinks: Vec<_> = sinks.iter().collect();
    // *counter += 1;
    // *counter %= sinks.len();
    // let sink = sinks[*counter];

    for (source_entity, source_anchor) in sources {
        if !matches!(source_anchor, Anchor::Wall(_)) {
            continue;
        }

        for tile_entity in tile_collections
            .iter_descendants(source_entity)
            .sorted_by_key(|e| match tile_query.get(*e) {
                Ok(slot) => slot.0,
                Err(_) => 0,
            })
        {
            for (sink_entity, sink_anchor) in sinks {
                if !matches!(sink_anchor, Anchor::Hand(_)) {
                    continue;
                }

                // the sink is a hand, and the descendants of sink are Tiles
                let descendants: Vec<_> = tile_collections.iter_descendants(sink_entity).collect();

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
                    let Ok(Slot(x)) = tile_query.get(descendant) else {
                        continue;
                    };
                    set.remove(&x);
                }

                commands
                    .entity(tile_entity)
                    .insert(Slot(set.into_iter().next().unwrap()));

                messages.write(TransferTile {
                    tile: tile_entity,
                    src: source_entity,
                    dest: sink_entity,
                });
                return;
            }
            info!("going to the draw state");
            next_state.set(LevelState::Draw);
            return;
        }
    }
    info!("going to the build wall state");
    next_state.set(LevelState::BuildWall);

    // If both players hands are at least size 14, we go to the draw state.

    // Otherwise, we maintain this state.
    // Play a tile into any hand of size < 14.
    // ...

    */

    let beans = model.wall.len() - 13;
    let mut player_hand = model.wall.split_off(beans);

    for i in 0..13 {
        messages.write(TileTransferMsg {
            start: TileLocation::Wall,
            end: TileLocation::Hand(Owner::Player, i),
            tile: player_hand[i],
        });
    }
    player_hand.sort();
    dbg!(&player_hand);

    let beans = model.wall.len() - 13;
    let mut enemy_hand = model.wall.split_off(beans);
    for i in 0..13 {
        messages.write(TileTransferMsg {
            start: TileLocation::Wall,
            end: TileLocation::Hand(Owner::AI, i),
            tile: enemy_hand[i],
        });
    }
    enemy_hand.sort();
    dbg!(&enemy_hand);

    for (mut actor, owner) in actors {
        match owner {
            Owner::Player => actor.hand = player_hand.clone(),
            Owner::AI => actor.hand = enemy_hand.clone(),
        }
    }

    next_state.set(LevelState::Draw);
}
