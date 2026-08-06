use std::collections::HashSet;

use crate::{
    card::{Card, Rank, Suit},
    errors::GameResult,
    id::CardId,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MeldType {
    Set { rank: Rank, suits: HashSet<Suit> },
    Run { suit: Suit, start: Rank, end: Rank },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Meld {
    meld_type: MeldType,
    cards: Vec<CardId>,
}

impl Meld {
    pub fn new_set(cards: &[Card]) -> GameResult<Self> {
        todo!()
    }

    pub fn new_run(cards: &[Card]) -> GameResult<Self> {
        todo!()
    }

    pub fn add_to_set(card: Card) -> GameResult<()> {
        todo!()
    }

    pub fn add_to_run(card: Card) -> GameResult<()> {
        todo!()
    }
}
