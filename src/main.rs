use bevy::prelude::*;
use mahjong::MahjongPlugin;

fn main() {
    App::new().add_plugins(MahjongPlugin).run();
}
