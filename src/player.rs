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
    pub full_deck: Deck,
    pub base_hp: i32,
    pub base_shield: i32,
    /// Damage dealt when stealing a tile
    pub call_damage: i32,
}

impl PlayerLoadout {
    /// Creates the loadout for the default player (name pending)
    pub fn default_player() -> Self {
        Self {
            full_deck: default_deck(),
            base_hp: 250,
            base_shield: 0,
            call_damage: 30,
        }
    }

    pub fn actor_state(&self) -> ActorState {
        ActorState {
            hp: self.base_hp,
            shield: self.base_shield,
            hand: Vec::new(),
            sets: Vec::new(),
        }
    }
}

/// The state of an "actor" (i.e., player or enemy)
#[derive(Component)]
pub struct ActorState {
    pub hp: i32,
    pub shield: i32,
    /// The actor's hand (13 tiles to start with)
    pub hand: Vec<TileKind>,
    /// Sets created by stealing
    /// This is a vec of vecs since each set is distinct
    pub sets: Vec<Vec<TileKind>>,
}

impl ActorState {
    pub fn default_enemy() -> Self {
        ActorState {
            hp: 100,
            shield: 0,
            hand: Vec::new(),
            sets: Vec::new(),
        }
    }
}
