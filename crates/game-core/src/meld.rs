use std::collections::HashSet;

use crate::{
    card::{Card, Rank, Suit, incr_rank, next_rank, prev_rank},
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
    num_joker_cards: usize,
}

impl Meld {
    pub fn new_set(cards: &[Card]) -> GameResult<Self> {
        let (rank, suits, num_joker_cards) = validate_set_cards(cards, true)?;
        let card_ids = validate_unique_card_ids(cards)?;
        Ok(Self {
            meld_type: MeldType::Set { rank, suits },
            card_ids,
            num_joker_cards,
        })
    }

    pub fn new_run(cards: &[Card]) -> GameResult<Self> {
        let (suit, start, end, num_joker_cards) = validate_run_cards(cards, true)?;
        let card_ids = validate_unique_card_ids(cards)?;
        Ok(Self {
            meld_type: MeldType::Run { suit, start, end },
            card_ids,
            num_joker_cards,
        })
    }

    pub fn add(&mut self, card: Card) -> GameResult<()> {
        if self.card_ids.contains(&card.id) {
            return error(MeldError::CardAlreadyInMeld);
        }
        match &mut self.meld_type {
            MeldType::Set { rank, suits } => {
                match card.face {
                    crate::card::CardFace::Standard {
                        rank: card_rank,
                        suit,
                    } => {
                        if card_rank != *rank {
                            return error(MeldError::SetMustHaveSameRank);
                        }
                        if suits.contains(&suit) {
                            return error(MeldError::SetMustHaveUniqueSuits);
                        }
                        if suits.len() + self.num_joker_cards + 1 > 4 {
                            return error(MeldError::SetCannotHaveMoreThanFourCards);
                        }
                        suits.insert(suit);
                    }
                    crate::card::CardFace::Joker => {
                        if suits.len() + self.num_joker_cards + 1 > 4 {
                            return error(MeldError::SetCannotHaveMoreThanFourCards);
                        }
                    }
                }
                self.card_ids.push(card.id);
                Ok(())
            }
            MeldType::Run { suit, start, end } => {
                match card.face {
                    crate::card::CardFace::Standard {
                        rank: card_rank,
                        suit: card_suit,
                    } => {
                        if card_suit != *suit {
                            return error(MeldError::RunMustHaveSameSuit);
                        }
                        if let Some(new_start) = prev_rank(*start)
                            && card_rank == new_start
                        {
                            *start = new_start;
                            self.card_ids.push(card.id);
                            return Ok(());
                        }
                        if let Some(new_end) = next_rank(*end)
                            && card_rank == new_end
                            && card_rank > *start
                        {
                            *end = new_end;
                            self.card_ids.push(card.id);
                            return Ok(());
                        }
                        if *end == Rank::King && card_rank == Rank::Ace {
                            *end = Rank::Ace;
                            self.card_ids.push(card.id);
                            return Ok(());
                        }
                        error(MeldError::RunMustHaveConsecutiveRanks)
                    }
                    crate::card::CardFace::Joker => {
                        // Joker can be added to either end of the run
                        if let Some(new_start) = prev_rank(*start) {
                            *start = new_start;
                            self.card_ids.push(card.id);
                            return Ok(());
                        }
                        if let Some(new_end) = next_rank(*end) {
                            *end = new_end;
                            self.card_ids.push(card.id);
                            return Ok(());
                        }
                        error(MeldError::MeldHasTooManyJokerCards)
                    }
                }
            }
        }
    }
}

fn error<T>(e: MeldError) -> GameResult<T> {
    Err(GameError::MeldError(e))
}

fn validate_set_cards(
    cards: &[Card],
    require_complete: bool,
) -> GameResult<(Rank, HashSet<Suit>, usize)> {
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

    Ok((ranks[0], suits, num_joker_cards))
}

fn validate_run_cards(
    cards: &[Card],
    require_complete: bool,
) -> GameResult<(Suit, Rank, Rank, usize)> {
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

    if !sorted_non_ace_ranks.is_empty() {
        let mut start = sorted_non_ace_ranks[0];
        let mut end = sorted_non_ace_ranks[0];
        let mut num_joker_cards = num_joker_cards;

        let mut remaining_jokers = num_joker_cards;
        for &rank in sorted_non_ace_ranks.iter().skip(1) {
            let d = rank as usize - end as usize;
            if d == 1 {
                end = rank;
            } else if d - 1 <= remaining_jokers {
                remaining_jokers -= d - 1;
                num_joker_cards += d - 1;
                end = rank;
            } else {
                return error(MeldError::RunMustHaveConsecutiveRanks);
            }
        }

        let mut num_unused_ace_ranks = num_ace_ranks;
        if num_unused_ace_ranks > 0 {
            let d = start as usize - Rank::Ace as usize;
            if d - 1 <= remaining_jokers {
                num_unused_ace_ranks -= 1;
                remaining_jokers -= d - 1;
                num_joker_cards += d - 1;
                start = Rank::Ace;
            }
        }
        if num_unused_ace_ranks > 0 {
            let d = (Rank::King as usize - end as usize) + 1;
            if d - 1 <= remaining_jokers {
                num_unused_ace_ranks -= 1;
                remaining_jokers -= d - 1;
                num_joker_cards += d - 1;
                end = Rank::Ace;
            }
        }

        while remaining_jokers > 0 {
            if let Some(next_rank) = next_rank(end) {
                end = next_rank;
                remaining_jokers -= 1;
            } else if let Some(prev_rank) = prev_rank(start) {
                start = prev_rank;
                remaining_jokers -= 1;
            } else {
                break;
            }
        }

        if remaining_jokers > 0 {
            return error(MeldError::MeldHasTooManyJokerCards);
        }

        if num_unused_ace_ranks > 0 {
            return error(MeldError::RunMustHaveConsecutiveRanks);
        }
        Ok((suits[0], start, end, num_joker_cards))
    } else if num_ace_ranks == 1 && num_joker_cards >= 2 {
        let start = Rank::Ace;
        match incr_rank(start, num_joker_cards) {
            Some(end) => Ok((suits[0], start, end, num_joker_cards)),
            None => error(MeldError::RunMustHaveConsecutiveRanks),
        }
    } else {
        error(MeldError::RunMustHaveConsecutiveRanks)
    }
}

fn validate_unique_card_ids(cards: &[Card]) -> GameResult<Vec<CardId>> {
    let card_ids: Vec<CardId> = cards.iter().map(|card| card.id).collect();
    let unique_card_ids: HashSet<CardId> = card_ids.iter().copied().collect();
    if unique_card_ids.len() == cards.len() {
        Ok(card_ids)
    } else {
        error(MeldError::NotEnoughCardsForMeld)
    }
}

#[cfg(test)]
mod tests {
    use super::{Meld, MeldType};
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

    #[test]
    fn new_run_additional_ace_in_beginning() {
        let cards = vec![
            card(Rank::Ace, Suit::Clubs),
            card(Rank::Three, Suit::Clubs),
            card(Rank::Four, Suit::Clubs),
            card(Rank::Five, Suit::Clubs),
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
    fn new_run_additional_ace_at_the_end() {
        let cards = vec![
            card(Rank::Ace, Suit::Clubs),
            card(Rank::Two, Suit::Clubs),
            card(Rank::Three, Suit::Clubs),
            card(Rank::Ace, Suit::Clubs),
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
    fn new_run_jokers1() {
        let cards = vec![
            card(Rank::Two, Suit::Clubs),
            card(Rank::Five, Suit::Clubs),
            card(Rank::Seven, Suit::Clubs),
            joker(),
            joker(),
            joker(),
        ];
        let result = Meld::new_run(&cards);
        assert!(result.is_ok());
    }

    #[test]
    fn add_extends_a_set_with_a_valid_card() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Diamonds),
            card(Rank::Seven, Suit::Hearts),
        ];
        let added_card = card(Rank::Seven, Suit::Spades);
        let Ok(mut meld) = Meld::new_set(&cards) else {
            panic!("the initial set should be valid");
        };

        let result = meld.add(added_card);

        assert!(result.is_ok());
        assert!(meld.card_ids.contains(&added_card.id));
        assert!(matches!(
            meld.meld_type,
            MeldType::Set { ref suits, .. } if suits.contains(&Suit::Spades)
        ));
    }

    #[test]
    fn add_extends_a_run_with_a_valid_card() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Eight, Suit::Clubs),
            card(Rank::Nine, Suit::Clubs),
        ];
        let added_card = card(Rank::Ten, Suit::Clubs);
        let Ok(mut meld) = Meld::new_run(&cards) else {
            panic!("the initial run should be valid");
        };

        let result = meld.add(added_card);

        assert!(result.is_ok());
        assert!(meld.card_ids.contains(&added_card.id));
        assert!(matches!(
            meld.meld_type,
            MeldType::Run {
                start: Rank::Seven,
                end: Rank::Ten,
                ..
            }
        ));
    }

    #[test]
    fn new_run_rejects_jokers_that_cannot_be_assigned_to_the_run() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Eight, Suit::Clubs),
            card(Rank::Nine, Suit::Clubs),
            joker(),
            joker(),
            joker(),
            joker(),
            joker(),
            joker(),
            joker(),
            joker(),
            joker(),
            joker(),
            joker(),
            joker(),
        ];

        let result = Meld::new_run(&cards);

        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::MeldHasTooManyJokerCards
            ))
        ));
    }

    #[test]
    fn new_meld_preserves_the_submitted_card_order() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Diamonds),
            card(Rank::Seven, Suit::Hearts),
        ];
        let expected_ids: Vec<CardId> = cards.iter().map(|card| card.id).collect();

        for _ in 0..32 {
            let Ok(meld) = Meld::new_set(&cards) else {
                panic!("the set should be valid");
            };
            assert_eq!(meld.card_ids, expected_ids);
        }
    }

    #[test]
    fn add_rejects_wrapping_a_high_ace_run_to_two() {
        let cards = vec![
            card(Rank::Queen, Suit::Clubs),
            card(Rank::King, Suit::Clubs),
            card(Rank::Ace, Suit::Clubs),
        ];
        let Ok(mut meld) = Meld::new_run(&cards) else {
            panic!("the initial high-ace run should be valid");
        };

        let result = meld.add(card(Rank::Two, Suit::Clubs));

        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::RunMustHaveConsecutiveRanks
            ))
        ));
    }

    #[test]
    fn add_rejects_a_fifth_card_when_a_set_contains_jokers() {
        let cards = vec![card(Rank::Seven, Suit::Clubs), joker(), joker()];
        let Ok(mut meld) = Meld::new_set(&cards) else {
            panic!("the initial set should be valid");
        };
        assert!(meld.add(card(Rank::Seven, Suit::Diamonds)).is_ok());

        let result = meld.add(card(Rank::Seven, Suit::Hearts));

        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::SetCannotHaveMoreThanFourCards
            ))
        ));
    }

    #[test]
    fn add_rejects_a_card_id_already_in_the_meld() {
        let duplicate = joker();
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Diamonds),
            duplicate,
        ];
        let Ok(mut meld) = Meld::new_set(&cards) else {
            panic!("the initial set should be valid");
        };

        let result = meld.add(duplicate);

        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::CardAlreadyInMeld
            ))
        ));
        assert_eq!(
            meld.card_ids
                .iter()
                .filter(|&&card_id| card_id == duplicate.id)
                .count(),
            1
        );
    }

    #[test]
    fn add_extends_a_run_with_a_high_ace() {
        let cards = vec![
            card(Rank::Jack, Suit::Clubs),
            card(Rank::Queen, Suit::Clubs),
            card(Rank::King, Suit::Clubs),
        ];
        let ace = card(Rank::Ace, Suit::Clubs);
        let Ok(mut meld) = Meld::new_run(&cards) else {
            panic!("the initial run should be valid");
        };

        let result = meld.add(ace);

        assert!(result.is_ok());
        assert!(meld.card_ids.contains(&ace.id));
        assert!(matches!(
            meld.meld_type,
            MeldType::Run {
                start: Rank::Jack,
                end: Rank::Ace,
                ..
            }
        ));
    }

    #[test]
    fn new_run_accepts_jokers_filling_both_ace_positions() {
        let mut cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Eight, Suit::Clubs),
            card(Rank::Nine, Suit::Clubs),
        ];
        cards.extend((0..11).map(|_| joker()));

        let result = Meld::new_run(&cards);

        assert!(result.is_ok());
    }

    #[test]
    fn equivalent_sets_have_a_stable_serialized_representation() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Diamonds),
            card(Rank::Seven, Suit::Hearts),
            card(Rank::Seven, Suit::Spades),
        ];
        let Ok(first_meld) = Meld::new_set(&cards) else {
            panic!("the set should be valid");
        };
        let Ok(expected) = serde_json::to_string(&first_meld) else {
            panic!("the meld should serialize");
        };

        for _ in 0..64 {
            let Ok(meld) = Meld::new_set(&cards) else {
                panic!("the set should be valid");
            };
            let Ok(serialized) = serde_json::to_string(&meld) else {
                panic!("the meld should serialize");
            };
            assert_eq!(serialized, expected);
        }
    }
}
