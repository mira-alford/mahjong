//! The player select screen
use bevy::prelude::*;

use crate::{GameState, player::PlayerLoadout};

pub fn player_select_plugin(app: &mut App) {
    app.insert_resource(PlayerLoadout::default_player())
        .add_systems(OnEnter(GameState::PlayerSelect), spawn_player_select)
        .add_systems(
            Update,
            player_button_system.run_if(in_state(GameState::PlayerSelect)),
        );
}

#[derive(Component)]
struct ChoosePlayerButton(PlayerLoadout);

const PLAYER_BUTTON_NORMAL: Color = Color::srgb_u8(100, 50, 50);
const PLAYER_BUTTON_HOVERED: Color = Color::srgb_u8(230, 75, 50);

fn spawn_player_select(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::PlayerSelect),
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            children![
                (
                    Text::new("Choose your loadout"),
                    TextFont {
                        font_size: 60.,
                        ..default()
                    }
                ),
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    children![(
                        Node {
                            width: px(300),
                            height: px(400),
                            ..default()
                        },
                        Button,
                        ChoosePlayerButton(PlayerLoadout::default_player()),
                        BackgroundColor(PLAYER_BUTTON_NORMAL),
                        children![Text::new("Default")]
                    )]
                ),
                Text::new("We only have one player right now. sorry")
            ]
        ),],
    ));
}

fn player_button_system(
    interactions: Query<
        (
            &Interaction,
            &ChoosePlayerButton,
            &mut Node,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_loadout: ResMut<PlayerLoadout>,
) {
    for (interaction, ChoosePlayerButton(loadout), mut node, mut colour) in interactions {
        match interaction {
            Interaction::Pressed => {
                *player_loadout = loadout.clone();
                next_state.set(GameState::Game);
            }
            Interaction::Hovered => {
                node.width = px(320);
                node.height = px(420);
                colour.0 = PLAYER_BUTTON_HOVERED;
            }
            Interaction::None => {
                node.width = px(300);
                node.height = px(400);
                colour.0 = PLAYER_BUTTON_NORMAL;
            }
        }
    }
}
