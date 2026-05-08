use bevy::{input_focus::InputFocus, prelude::*};

use crate::GameState;

pub fn title_menu_plugin(app: &mut App) {
    app.init_resource::<InputFocus>()
        .add_systems(OnEnter(GameState::TitleMenu), spawn_title_menu)
        .add_systems(
            Update,
            menu_button_system.run_if(in_state(GameState::TitleMenu)),
        );
}

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Exit,
}

fn spawn_title_menu(mut commands: Commands) {
    let button_node = Node {
        width: px(300),
        height: px(65),
        margin: UiRect::all(px(20)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    commands.spawn((
        DespawnOnExit(GameState::TitleMenu),
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
                    Text::new("MAHJONG GAME"),
                    TextFont {
                        font_size: 60.,
                        ..default()
                    }
                ),
                (
                    Button,
                    BackgroundColor(Color::srgb_u8(0, 255, 0)),
                    MenuButtonAction::Play,
                    button_node.clone(),
                    children![Text::new("Play")]
                ),
                (
                    Button,
                    BackgroundColor(Color::srgb_u8(255, 0, 0)),
                    MenuButtonAction::Exit,
                    button_node.clone(),
                    children![Text::new("Quit")]
                )
            ]
        ),],
    ));
}

fn menu_button_system(
    interactions: Query<(&Interaction, &MenuButtonAction), (Changed<Interaction>, With<Button>)>,
    mut exit: MessageWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, action) in interactions {
        if *interaction == Interaction::Pressed {
            match action {
                MenuButtonAction::Play => {
                    next_state.set(GameState::Game);
                }
                MenuButtonAction::Exit => {
                    exit.write(AppExit::Success);
                }
            }
        }
    }
}
