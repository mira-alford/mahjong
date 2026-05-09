//! Stuff to do with the player's loadout - their deck, their items, etc

use crate::tile::kind::{Dragon, Honor, Suit, TileKind, Wind};
use bevy::prelude::*;

type Deck = Vec<TileKind>;

// Creates the standard mahjong deck
fn default_deck() -> Deck {
    let mut res = Vec::new();

    for _ in 0..4 {
        for num in 1..=9 {
            res.push(TileKind::Suit(Suit::Characters(num)));
            res.push(TileKind::Suit(Suit::Bamboo(num)));
            res.push(TileKind::Suit(Suit::Circle(num)));
        }

        for dir in [Wind::East, Wind::South, Wind::West, Wind::North] {
            res.push(TileKind::Honor(Honor::Wind(dir)));
        }

        for col in [Dragon::Red, Dragon::Green, Dragon::White] {
            res.push(TileKind::Honor(Honor::Dragon(col)));
        }
    }

    // This doesn't really matter I guess? We will shuffle before playing anyway
    res.sort();
    res
}

#[derive(Resource, Clone, Debug)]
pub struct PlayerLoadout {
    // The deck the player starts with each round
    full_deck: Deck,
    base_hp: i32,
    base_shield: i32,
}

impl PlayerLoadout {
    /// Creates the loadout for the default player (name pending)
    pub fn default_player() -> Self {
        Self {
            full_deck: default_deck(),
            base_hp: 250,
            base_shield: 0,
        }
    }
}
