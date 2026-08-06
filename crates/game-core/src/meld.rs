use std::collections::HashSet;

use crate::{
    card::{Card, Rank, Suit},
    errors::{GameError, GameResult, MeldError},
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
        let (rank, suits) = validate_set_cards(cards, true)?;
        let card_ids = validate_unique_card_ids(cards)?;
        Ok(Self {
            meld_type: MeldType::Set { rank, suits },
            card_ids,
        })
    }

    pub fn new_run(cards: &[Card]) -> GameResult<Self> {
        let (suit, start, end) = validate_run_cards(cards, true)?;
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

fn error<T>(e: MeldError) -> GameResult<T> {
    Err(GameError::MeldError(e))
}

fn validate_set_cards(cards: &[Card], require_complete: bool) -> GameResult<(Rank, HashSet<Suit>)> {
    if require_complete && cards.len() < 3 {
        return error(MeldError::NotEnoughCardsForMeld);
    }

    let num_joker_cards = cards
        .iter()
        .filter(|card| matches!(card.face, crate::card::CardFace::Joker))
        .count();
    let non_joker_cards: Vec<&Card> = cards
        .iter()
        .filter(|card| !matches!(card.face, crate::card::CardFace::Joker))
        .collect();
    if non_joker_cards.is_empty() {
        return error(MeldError::MeldMustHaveNonJokerCards);
    }
    let ranks: HashSet<Rank> = non_joker_cards
        .iter()
        .map(|card| match card.face {
            crate::card::CardFace::Standard { rank, .. } => rank,
            crate::card::CardFace::Joker => unreachable!(),
        })
        .collect();
    let suits: HashSet<Suit> = non_joker_cards
        .iter()
        .map(|card| match card.face {
            crate::card::CardFace::Standard { suit, .. } => suit,
            crate::card::CardFace::Joker => unreachable!(),
        })
        .collect();

    if ranks.len() != 1 {
        return error(MeldError::SetMustHaveSameRank);
    }

    if suits.len() != non_joker_cards.len() {
        return error(MeldError::SetMustHaveUniqueSuits);
    }

    if suits.len() + num_joker_cards > 4 {
        return error(MeldError::SetMustHaveUniqueSuits);
    }

    Ok((*ranks.iter().next().unwrap(), suits))
}

fn validate_run_cards(cards: &[Card], require_complete: bool) -> GameResult<(Suit, Rank, Rank)> {
    todo!()
}

fn validate_unique_card_ids(cards: &[Card]) -> GameResult<Vec<CardId>> {
    let card_ids: HashSet<CardId> = cards.iter().map(|card| card.id).collect();
    if card_ids.len() != cards.len() {
        error(MeldError::NotEnoughCardsForMeld)
    } else {
        Ok(card_ids.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::Meld;
    use crate::{
        card::{Card, CardFace, Rank, Suit},
        errors::GameError,
        id::CardId,
    };

    fn card(rank: Rank, suit: Suit) -> Card {
        Card {
            id: CardId::new(),
            face: CardFace::Standard { rank, suit },
        }
    }

    #[test]
    fn new_set_requires_at_least_three_cards() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Hearts),
        ];
        let result = Meld::new_set(&cards);
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::NotEnoughCardsForMeld
            ))
        ));
    }
}
