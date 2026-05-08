use bevy::prelude::*;

use crate::GameState;

pub fn title_menu_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::TitleMenu), spawn_title_menu);
}

fn spawn_title_menu(mut commands: Commands, assets: Res<AssetServer>) {
    let icon = assets.load("icon.svg");

    commands.spawn((
        DespawnOnExit(GameState::TitleMenu),
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        children![
            ImageNode::new(icon),
            Node {
                width: Val::Px(200.),
                ..Default::default()
            }
        ],
    ));
}
