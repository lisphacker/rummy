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
    card_ids: Vec<CardId>,
}

impl Meld {
    pub fn new_set(cards: &[Card]) -> GameResult<Self> {
        let (rank, suits) = validate_set_cards(cards)?;
        let card_ids = validate_unique_card_ids(cards)?;
        Ok(Self {
            meld_type: MeldType::Set { rank, suits },
            card_ids,
        })
    }

    pub fn new_run(cards: &[Card]) -> GameResult<Self> {
        let (suit, start, end) = validate_run_cards(cards)?;
        let card_ids = validate_unique_card_ids(cards)?;
        Ok(Self {
            meld_type: MeldType::Run { suit, start, end },
            card_ids,
        })
    }

    pub fn add(&mut self, card: Card) -> GameResult<()> {
        match &mut self.meld_type {
            MeldType::Set { .. } => self.add_to_set(card),
            MeldType::Run { .. } => self.add_to_run(card),
        }
    }

    fn add_to_set(&mut self, card: Card) -> GameResult<()> {
        todo!()
    }

    fn add_to_run(&mut self, card: Card) -> GameResult<()> {
        todo!()
    }
}

fn validate_set_cards(cards: &[Card]) -> GameResult<(Rank, HashSet<Suit>)> {
    todo!()
}

fn validate_run_cards(cards: &[Card]) -> GameResult<(Suit, Rank, Rank)> {
    todo!()
}

fn validate_unique_card_ids(cards: &[Card]) -> GameResult<Vec<CardId>> {
    todo!()
}
