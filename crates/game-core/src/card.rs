use crate::id::CardId;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    pub const ALL: [Self; 4] = [Self::Clubs, Self::Diamonds, Self::Hearts, Self::Spades];

    pub fn iter() -> impl ExactSizeIterator<Item = Self> {
        Self::ALL.into_iter()
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl Rank {
    pub const ALL: [Self; 13] = [
        Self::Ace,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::Ten,
        Self::Jack,
        Self::Queen,
        Self::King,
    ];

    pub fn iter() -> impl ExactSizeIterator<Item = Self> {
        Self::ALL.into_iter()
    }
}

fn rank_to_usize(rank: Rank) -> usize {
    match rank {
        Rank::Ace => 0,
        Rank::Two => 1,
        Rank::Three => 2,
        Rank::Four => 3,
        Rank::Five => 4,
        Rank::Six => 5,
        Rank::Seven => 6,
        Rank::Eight => 7,
        Rank::Nine => 8,
        Rank::Ten => 9,
        Rank::Jack => 10,
        Rank::Queen => 11,
        Rank::King => 12,
    }
}

fn usize_to_rank(n: usize) -> Option<Rank> {
    match n {
        0 => Some(Rank::Ace),
        1 => Some(Rank::Two),
        2 => Some(Rank::Three),
        3 => Some(Rank::Four),
        4 => Some(Rank::Five),
        5 => Some(Rank::Six),
        6 => Some(Rank::Seven),
        7 => Some(Rank::Eight),
        8 => Some(Rank::Nine),
        9 => Some(Rank::Ten),
        10 => Some(Rank::Jack),
        11 => Some(Rank::Queen),
        12 => Some(Rank::King),
        _ => None,
    }
}

#[must_use]
pub fn incr_rank(rank: Rank, n: usize) -> Option<Rank> {
    usize_to_rank(rank_to_usize(rank) + n)
}

#[must_use]
pub fn prev_rank(rank: Rank) -> Option<Rank> {
    usize_to_rank(rank_to_usize(rank).checked_sub(1)?)
}

#[must_use]
pub fn next_rank(rank: Rank) -> Option<Rank> {
    usize_to_rank(rank_to_usize(rank) + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CardFace {
    Standard { rank: Rank, suit: Suit },
    Joker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Card {
    pub id: CardId,
    pub face: CardFace,
}

impl Card {
    #[must_use]
    pub const fn standard(id: CardId, suit: Suit, rank: Rank) -> Self {
        Self {
            id,
            face: CardFace::Standard { rank, suit },
        }
    }

    #[must_use]
    pub const fn joker(id: CardId) -> Self {
        Self {
            id,
            face: CardFace::Joker,
        }
    }
}
