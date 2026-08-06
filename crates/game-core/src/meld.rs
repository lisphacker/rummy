use std::collections::HashSet;

use crate::{
    card::{Card, Rank, Suit, incr_rank},
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

    fn add_to_set(&mut self, _card: Card) -> GameResult<()> {
        todo!()
    }

    fn add_to_run(&mut self, _card: Card) -> GameResult<()> {
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

    let ranks: Vec<Rank> = non_joker_cards
        .iter()
        .map(|card| match card.face {
            crate::card::CardFace::Standard { rank, .. } => rank,
            crate::card::CardFace::Joker => unreachable!(),
        })
        .collect::<HashSet<Rank>>()
        .into_iter()
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
        return error(MeldError::SetHasTooManyJokers);
    }

    Ok((ranks[0], suits))
}

fn validate_run_cards(cards: &[Card], require_complete: bool) -> GameResult<(Suit, Rank, Rank)> {
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

    let ranks: Vec<Rank> = non_joker_cards
        .iter()
        .map(|card| match card.face {
            crate::card::CardFace::Standard { rank, .. } => rank,
            crate::card::CardFace::Joker => unreachable!(),
        })
        // .collect::<HashSet<Rank>>()
        // .into_iter()
        .collect();

    let suits: Vec<Suit> = non_joker_cards
        .iter()
        .map(|card| match card.face {
            crate::card::CardFace::Standard { suit, .. } => suit,
            crate::card::CardFace::Joker => unreachable!(),
        })
        .collect::<HashSet<Suit>>()
        .into_iter()
        .collect();

    if suits.len() != 1 {
        return error(MeldError::RunMustHaveSameSuit);
    }

    let num_ace_ranks = ranks.iter().filter(|&&rank| rank == Rank::Ace).count();
    if num_ace_ranks > 2 {
        return error(MeldError::RankHasTooManyAces);
    }

    let non_ace_ranks: Vec<Rank> = ranks
        .iter()
        .filter(|&&rank| rank != Rank::Ace)
        .copied()
        .collect::<HashSet<Rank>>()
        .into_iter()
        .collect();
    let mut sorted_non_ace_ranks = non_ace_ranks.clone();
    sorted_non_ace_ranks.sort();

    if non_ace_ranks.len() + num_ace_ranks != non_joker_cards.len() {
        // Must have unique ranks
        return error(MeldError::RunMustHaveConsecutiveRanks);
    }

    if sorted_non_ace_ranks.len() >= 1 {
        let mut start = sorted_non_ace_ranks[0];
        let mut end = sorted_non_ace_ranks[0];

        let mut remaining_jokers = num_joker_cards;
        for &rank in sorted_non_ace_ranks.iter().skip(1) {
            let d = rank as u8 - end as u8;
            if d == 1 {
                end = rank;
            } else if d - 1 <= remaining_jokers as u8 {
                remaining_jokers -= (d - 1) as usize;
                end = rank;
            } else {
                return error(MeldError::RunMustHaveConsecutiveRanks);
            }
        }

        let mut num_unused_ace_ranks = num_ace_ranks;
        if num_unused_ace_ranks > 0 {
            let d = start as u8 - Rank::Ace as u8;
            if d - 1 <= remaining_jokers as u8 {
                num_unused_ace_ranks -= 1;
                remaining_jokers -= (d - 1) as usize;
                start = Rank::Ace;
            }
        }
        if num_unused_ace_ranks > 0 {
            let d = (Rank::King as u8 - end as u8) + 1;
            if d - 1 <= remaining_jokers as u8 {
                num_unused_ace_ranks -= 1;
                // remaining_jokers -= (d - 1) as usize;
                end = Rank::Ace;
            }
        }
        if num_unused_ace_ranks > 0 {
            return error(MeldError::RunMustHaveConsecutiveRanks);
        }
        Ok((suits[0], start, end))
    } else {
        if num_ace_ranks == 1 && num_joker_cards >= 2 {
            let start = Rank::Ace;
            match incr_rank(start, num_joker_cards) {
                Some(end) => Ok((suits[0], start, end)),
                None => error(MeldError::RunMustHaveConsecutiveRanks),
            }
        } else {
            error(MeldError::RunMustHaveConsecutiveRanks)
        }
    }
}

fn validate_unique_card_ids(cards: &[Card]) -> GameResult<Vec<CardId>> {
    let card_ids: HashSet<CardId> = cards.iter().map(|card| card.id).collect();
    if card_ids.len() == cards.len() {
        Ok(card_ids.into_iter().collect())
    } else {
        error(MeldError::NotEnoughCardsForMeld)
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

    fn joker() -> Card {
        Card {
            id: CardId::new(),
            face: CardFace::Joker,
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

    #[test]
    fn new_set_has_no_non_joker_cards() {
        let cards = vec![joker(), joker(), joker()];
        let result = Meld::new_set(&cards);
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::MeldMustHaveNonJokerCards
            ))
        ));
    }

    #[test]
    fn new_set_has_non_unique_ranks() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Eight, Suit::Hearts),
            card(Rank::Seven, Suit::Diamonds),
        ];
        let result = Meld::new_set(&cards);
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::SetMustHaveSameRank
            ))
        ));
    }

    #[test]
    fn new_set_has_non_unique_suits() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Diamonds),
        ];
        let result = Meld::new_set(&cards);
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::SetMustHaveUniqueSuits
            ))
        ));
    }

    #[test]
    fn new_set_has_two_many_jokers() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Hearts),
            card(Rank::Seven, Suit::Diamonds),
            joker(),
            joker(),
        ];
        let result = Meld::new_set(&cards);
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::SetHasTooManyJokers
            ))
        ));
    }

    #[test]
    fn new_run_requires_at_least_three_cards() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Eight, Suit::Clubs),
        ];
        let result = Meld::new_run(&cards);
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::NotEnoughCardsForMeld
            ))
        ));
    }

    #[test]
    fn new_run_has_more_than_one_suit() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Eight, Suit::Hearts),
            card(Rank::Nine, Suit::Clubs),
        ];
        let result = Meld::new_run(&cards);
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::RunMustHaveSameSuit
            ))
        ));
    }

    #[test]
    fn new_run_has_repeating_ranks() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Eight, Suit::Clubs),
        ];
        let result = Meld::new_run(&cards);
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::RunMustHaveConsecutiveRanks
            ))
        ));
    }

    #[test]
    fn new_run_has_two_many_aces() {
        let cards = vec![
            card(Rank::Ace, Suit::Clubs),
            card(Rank::Two, Suit::Clubs),
            card(Rank::Three, Suit::Clubs),
            card(Rank::Ace, Suit::Clubs),
            card(Rank::Ace, Suit::Clubs),
        ];
        let result = Meld::new_run(&cards);
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::RankHasTooManyAces
            ))
        ));
    }
}
