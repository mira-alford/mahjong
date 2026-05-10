use bevy::prelude::*;

use crate::GameState;

#[derive(Component)]
struct TuteStuff;

#[derive(Component)]
struct TuteRoot;

pub fn tutorial_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Tutorial), spawn_tutorial_scene)
        .add_systems(
            Update,
            advance_tutorial.run_if(in_state(GameState::Tutorial)),
        );
}

fn advance_tutorial(
    mut commands: Commands,
    tute_stuff: Query<Entity, With<TuteStuff>>,
    root: Query<Entity, With<TuteRoot>>,
    mut step: Local<usize>,
    mut state: ResMut<NextState<GameState>>,
    minput: Res<ButtonInput<MouseButton>>,
    kbinput: Res<ButtonInput<KeyCode>>,
    assets: Res<AssetServer>,
) {
    if minput.just_pressed(MouseButton::Left) || kbinput.just_pressed(KeyCode::Space) {
        *step += 1;

        for e in tute_stuff {
            commands.entity(e).despawn();
        }

        let root = root.single().unwrap();

        let next_item = match *step {
            1 => commands.spawn((TuteStuff, Text::new("You're finally awake"))),
            2 => commands.spawn((TuteStuff, Text::new("Have you ever played MAHJONG?"))),
            3 => commands.spawn((
                TuteStuff,
                Node::default(),
                children![
                    Text::new("Are you ready to play mahjong "),
                    (
                        Text::new("to the death??"),
                        TextColor(Color::srgb(1., 0., 0.))
                    )
                ],
            )),
            4 => commands.spawn((TuteStuff, Text::new("No? Well you'd better listen up."))),
            5 => commands.spawn((
                TuteStuff,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    Text::new("Mahjong is a game played with a deck of tiles. There are 3 suits of numbers 1-9 (circles, bamboo, characters), four winds (N,E,S,W) and three dragons (red, green, white)"),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        children![
                            ImageNode::new(assets.load("sprites/oneman.png")),
                            ImageNode::new(assets.load("sprites/onepin.png")),
                            ImageNode::new(assets.load("sprites/onesou.png")),
                            ImageNode::new(assets.load("sprites/ton.png")),
                            ImageNode::new(assets.load("sprites/shaa.png")),
                            Text::new("Etc"),
                        ]
                    ),
                ]
            )),
            6 => commands.spawn((TuteStuff, Text::new("You have a hand of tiles and you need to try to make sets"))),
            7 => commands.spawn((
                TuteStuff,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    Text::new("A set can be a three of a kind..."),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        children![
                            ImageNode::new(assets.load("sprites/oneman.png")),
                            ImageNode::new(assets.load("sprites/oneman.png")),
                            ImageNode::new(assets.load("sprites/oneman.png")),
                        ]
                    ),
                ]
            )),
            8 => commands.spawn((
                TuteStuff,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    Text::new("Or a pair..."),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        children![
                            ImageNode::new(assets.load("sprites/ton.png")),
                            ImageNode::new(assets.load("sprites/ton.png")),
                        ]
                    ),
                ]
            )),
            9 => commands.spawn((
                TuteStuff,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                children![
                    Text::new("Or a sequence of 3 consecutive number tiles"),
                    (
                        Node {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        children![
                            ImageNode::new(assets.load("sprites/onesou.png")),
                            ImageNode::new(assets.load("sprites/twosou.png")),
                            ImageNode::new(assets.load("sprites/threesou.png")),
                        ]
                    ),
                ]
            )),
            10 => commands.spawn((TuteStuff, Text::new("You have to draw and discard tiles until you make a hand of 4 sets of 3 and one pair"))),
            11 => commands.spawn((TuteStuff, Text::new("It gets much more complicated than that, but..."))),
            12 => commands.spawn((TuteStuff, Text::new("I'm sure you'll be fine. Anyway, you're in God's hands now. Forget I said that"))),
            13 => commands.spawn((TuteStuff, Text::new("Good luck!"))),
            _ => {
                state.set(GameState::PlayerSelect);
                return;
            }
        }
        .id();

        commands.entity(root).add_child(next_item);
    }
}

fn spawn_tutorial_scene(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::Tutorial),
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        TuteRoot,
        children![(
            TuteStuff,
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            children![Text::new("Hey")],
        )],
    ));
}
