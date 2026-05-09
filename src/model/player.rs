//! Stuff to do with the player's loadout - their deck, their items, etc

use crate::{
    events::TileLocation,
    tile::kind::{Dragon, Honor, Suit, TileKind, Wind},
};
use bevy::prelude::*;

type Deck = Vec<TileKind>;

// Creates the standard mahjong deck
fn default_deck() -> Deck {
    let mut res = Vec::new();

    for _ in 0..4 {
        for num in 1..=9 {
            res.push(TileKind::Number(Suit::Characters, num));
            res.push(TileKind::Number(Suit::Bamboo, num));
            res.push(TileKind::Number(Suit::Circle, num));
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

    pub fn actor_state(&self, hand: Vec<TileKind>) -> ActorState {
        ActorState {
            hp: self.base_hp,
            max_hp: self.base_hp,
            shield: self.base_shield,
            hand,
            discard: Vec::new(),
            sets: Vec::new(),
            drawn_tile: None,
        }
    }
}

/// The state of an "actor" (i.e., player or enemy)
#[derive(Component)]
pub struct ActorState {
    pub hp: i32,
    pub max_hp: i32,
    pub shield: i32,
    /// The actor's hand (13 tiles to start with)
    pub hand: Vec<TileKind>,
    pub discard: Vec<TileKind>,
    /// Sets created by stealing
    /// This is a vec of vecs since each set is distinct
    pub sets: Vec<Vec<TileKind>>,
    /// The tile that has just been drawn from the wall (if any)
    pub drawn_tile: Option<TileKind>,
}

/// A "set of tiles".
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TileSet {
    Sequence { suit: Suit, lowest_number: u8 },
    Triple(TileKind),
    Pair(TileKind),
}

impl ActorState {
    pub fn default_enemy(hand: Vec<TileKind>) -> Self {
        ActorState {
            hp: 100,
            max_hp: 100,
            shield: 0,
            hand,
            discard: Vec::new(),
            sets: Vec::new(),
            drawn_tile: None,
        }
    }

    pub fn draw_tile(&mut self, tile: TileKind) {
        assert!(self.drawn_tile.is_none());
        self.drawn_tile = Some(tile);
    }

    /// Returns a list of sets that can be made from stealing the tile that was just discarded.
    pub fn possible_steals(&self, discarded_tile: TileKind) -> Vec<TileSet> {
        // THIS CODE SUCKS ASS BUT I DONT HAVE THE TIME TO MAKE IT GOOD
        let mut res = vec![];

        if self
            .hand
            .iter()
            .filter(|tile| **tile == discarded_tile)
            .count()
            >= 2
        {
            res.push(TileSet::Triple(discarded_tile));
        }

        if let TileKind::Number(suit, num) = discarded_tile {
            if num <= 7 {
                if self.hand.contains(&TileKind::Number(suit, num + 1))
                    && self.hand.contains(&TileKind::Number(suit, num + 2))
                {
                    res.push(TileSet::Sequence {
                        suit,
                        lowest_number: num,
                    });
                }
            }

            if num <= 8 && num >= 2 {
                if self.hand.contains(&TileKind::Number(suit, num - 1))
                    && self.hand.contains(&TileKind::Number(suit, num + 1))
                {
                    res.push(TileSet::Sequence {
                        suit,
                        lowest_number: num - 1,
                    });
                }
            }

            if num >= 3 {
                if self.hand.contains(&TileKind::Number(suit, num - 2))
                    && self.hand.contains(&TileKind::Number(suit, num - 1))
                {
                    res.push(TileSet::Sequence {
                        suit,
                        lowest_number: num - 2,
                    });
                }
            }
        }

        res
    }

    /// Returns a bool indicating if the player can construct a winning hand with the tiles they
    /// hold (including the drawn tile - if it is None, this function will return false).
    ///
    /// This function is not correct. The reason it is not correct is because a correct solution
    /// would cause [this](https://github.com/bevyengine/bevy/issues/16002) issue.
    ///
    /// fucking bevy moment
    pub fn has_winning_hand(&self) -> bool {
        let Some(drawn) = self.drawn_tile else {
            return false;
        };
        let mut hand = self.hand.clone();
        hand.push(drawn);
        hand.sort();

        let mut sets = Vec::new();

        loop {
            if hand.len() == 0 {
                break;
            }

            if hand.len() == 1 {
                return false;
            }

            if hand.len() >= 3 {
                // Attempt to take a triple, then attempt to take a set
                if hand[0] == hand[1] && hand[1] == hand[2] {
                    sets.push(TileSet::Triple(hand[0]));
                    for _ in 0..3 {
                        hand.remove(0);
                    }
                    continue;
                }

                if let TileKind::Number(s1, n1) = hand[0]
                    && let TileKind::Number(s2, n2) = hand[1]
                    && let TileKind::Number(s3, n3) = hand[2]
                {
                    if s1 == s2 && s2 == s3 && n1 == n2 - 1 && n2 == n3 - 1 {
                        sets.push(TileSet::Sequence {
                            suit: s1,
                            lowest_number: n1,
                        });
                        for _ in 0..3 {
                            hand.remove(0);
                        }
                        continue;
                    }
                }
            }

            if hand.len() >= 2 {
                if hand[0] == hand[1] {
                    sets.push(TileSet::Pair(hand[0]));
                    for _ in 0..2 {
                        hand.remove(0);
                    }
                    continue;
                }
            }
        }

        let mut shape = sets
            .iter()
            .map(|s| match s {
                TileSet::Sequence { .. } => 3,
                TileSet::Triple(..) => 3,
                TileSet::Pair(..) => 2,
            })
            .collect::<Vec<i32>>();

        shape.sort();
        (shape == vec![2, 3, 3, 3, 3]) || (shape == vec![2, 2, 2, 2, 2, 2, 2])
    }

    /// Discards a tile from the hand, or from the drawn tile (if it exists).
    ///
    /// The location should not be Drawn if there is no drawn tile.
    pub fn discard_tile(&mut self, location: TileLocation) {
        match location {
            TileLocation::Draw(_) => {
                if let Some(x) = self.drawn_tile.take() {
                    self.discard.push(x)
                }
            }
            TileLocation::Hand(_, ix) => {
                self.discard.push(self.hand.remove(ix));
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        model::player::PlayerLoadout,
        tile::kind::{Dragon, Honor, Suit, TileKind},
    };

    #[test]
    fn test_winning_hand() {
        let mut player = PlayerLoadout::default_player().actor_state(vec![
            TileKind::Number(Suit::Bamboo, 1),
            TileKind::Number(Suit::Bamboo, 1),
            TileKind::Number(Suit::Bamboo, 1),
            TileKind::Number(Suit::Circle, 1),
            TileKind::Number(Suit::Circle, 1),
            TileKind::Number(Suit::Circle, 1),
            TileKind::Number(Suit::Bamboo, 4),
            TileKind::Number(Suit::Bamboo, 5),
            TileKind::Number(Suit::Bamboo, 6),
            TileKind::Honor(Honor::Dragon(Dragon::Red)),
            TileKind::Honor(Honor::Dragon(Dragon::Red)),
            TileKind::Honor(Honor::Dragon(Dragon::Red)),
            TileKind::Honor(Honor::Dragon(Dragon::White)),
        ]);

        player.draw_tile(TileKind::Honor(Honor::Dragon(Dragon::White)));

        assert!(player.has_winning_hand());
    }
}
