/// enums for the different kind of tiles avalible, TileKind is the root

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TileKind {
    Number(Suit, u8),
    Honor(Honor),
}

/// suits each with associated number between 1 and 9
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    Characters,
    Circle,
    Bamboo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Honor {
    Wind(Wind),
    Dragon(Dragon),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Wind {
    East,
    South,
    West,
    North,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Dragon {
    Red,
    Green,
    White,
}
