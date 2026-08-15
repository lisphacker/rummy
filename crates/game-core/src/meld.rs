use crate::{
    card::{Card, Rank, Suit, incr_rank, next_rank, prev_rank},
    errors::{GameError, GameResult, MeldError},
    id::CardId,
    rules::config::MeldRulesConfig,
};
use std::collections::BTreeSet;
use std::collections::HashSet;

fn all_unique<T: Eq + std::hash::Hash>(items: &[T]) -> bool {
    let unique_items: HashSet<&T> = items.iter().collect();
    unique_items.len() == items.len()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeldType {
    Set { rank: Rank, suits: BTreeSet<Suit> },
    Run { suit: Suit, start: Rank, end: Rank },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meld {
    meld_type: MeldType,
    card_ids: Vec<CardId>,
    num_joker_cards: usize,
}

impl Meld {
    pub fn new_set(cards: &[Card], rules: &MeldRulesConfig) -> GameResult<Self> {
        let (rank, suits, num_joker_cards) = validate_set_cards(cards, rules, true)?;
        let card_ids = validate_unique_card_ids(cards)?;
        Ok(Self {
            meld_type: MeldType::Set { rank, suits },
            card_ids,
            num_joker_cards,
        })
    }

    pub fn new_run(cards: &[Card], rules: &MeldRulesConfig) -> GameResult<Self> {
        let (suit, start, end, num_joker_cards) = validate_run_cards(cards, rules, true)?;
        let card_ids = validate_unique_card_ids(cards)?;
        Ok(Self {
            meld_type: MeldType::Run { suit, start, end },
            card_ids,
            num_joker_cards,
        })
    }
}

fn error<T>(e: MeldError) -> GameResult<T> {
    Err(GameError::MeldError(e))
}

fn validate_set_cards(
    cards: &[Card],
    rules: &MeldRulesConfig,
    require_complete: bool,
) -> GameResult<(Rank, BTreeSet<Suit>, usize)> {
    if require_complete && cards.len() < rules.minimum_meld_size() {
        return error(MeldError::NotEnoughCardsForMeld);
    }

    let num_joker_cards = cards
        .iter()
        .filter(|card| matches!(card.face, crate::card::CardFace::Joker))
        .count();
    if num_joker_cards > 0 && !rules.allow_jokers() {
        return error(MeldError::JokersNotAllowed);
    }
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

    let suits: BTreeSet<Suit> = non_joker_cards
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
    rules: &MeldRulesConfig,
    require_complete: bool,
) -> GameResult<(Suit, Rank, Rank, usize)> {
    if require_complete && cards.len() < rules.minimum_meld_size() {
        return error(MeldError::NotEnoughCardsForMeld);
    }

    let num_joker_cards = cards
        .iter()
        .filter(|card| matches!(card.face, crate::card::CardFace::Joker))
        .count();
    if num_joker_cards > 0 && !rules.allow_jokers() {
        return error(MeldError::JokersNotAllowed);
    }
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
        .collect();

    let suits: Vec<Suit> = non_joker_cards
        .iter()
        .map(|card| match card.face {
            crate::card::CardFace::Standard { suit, .. } => suit,
            crate::card::CardFace::Joker => unreachable!(),
        })
        .collect::<BTreeSet<Suit>>()
        .into_iter()
        .collect();

    if suits.len() != 1 {
        return error(MeldError::RunMustHaveSameSuit);
    }

    let num_ace_ranks = ranks.iter().filter(|&&rank| rank == Rank::Ace).count();
    if num_ace_ranks > 1 {
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

    if sorted_non_ace_ranks.is_empty() {
        if num_ace_ranks == 1 && num_joker_cards >= 2 {
            let start = Rank::Ace;
            match incr_rank(start, num_joker_cards) {
                Some(end) => Ok((suits[0], start, end, num_joker_cards)),
                None => error(MeldError::MeldHasTooManyJokerCards),
            }
        } else {
            return error(MeldError::NotEnoughCardsForMeld);
        }
    } else {
        let mut start = sorted_non_ace_ranks[0];
        let mut end = sorted_non_ace_ranks[0];

        let mut remaining_jokers = num_joker_cards;
        for &rank in sorted_non_ace_ranks.iter().skip(1) {
            let d = rank as usize - end as usize;
            if d == 1 {
                end = rank;
            } else if d - 1 <= remaining_jokers {
                remaining_jokers -= d - 1;
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
                start = Rank::Ace;
            }
        }
        if num_unused_ace_ranks > 0 {
            let d = (Rank::King as usize - end as usize) + 1;
            if d - 1 <= remaining_jokers {
                num_unused_ace_ranks -= 1;
                remaining_jokers -= d - 1;
                end = Rank::Ace;
            }
        }

        while remaining_jokers > 0 {
            if start == Rank::Ace && end == Rank::Ace {
                break;
            } else if let Some(next_rank) = next_rank(end)
                && next_rank > start
            {
                end = next_rank;
                remaining_jokers -= 1;
            } else if let Some(prev_rank) = prev_rank(start)
                && prev_rank < end
            {
                start = prev_rank;
                remaining_jokers -= 1;
            } else if end == Rank::King {
                end = Rank::Ace;
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
    use super::Meld;
    use crate::{
        card::{Card, CardFace, Rank, Suit},
        errors::GameError,
        id::CardId,
        rules::config::MeldRulesConfig,
    };

    fn rules() -> MeldRulesConfig {
        MeldRulesConfig::new(3, true)
    }

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
        let result = Meld::new_set(&cards, &rules());
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
        let result = Meld::new_set(&cards, &rules());
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::MeldMustHaveNonJokerCards
            ))
        ));
    }

    #[test]
    fn basic_rummy_rejects_jokers_in_melds() {
        let cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Seven, Suit::Diamonds),
            joker(),
        ];

        let result = Meld::new_set(&cards, &MeldRulesConfig::basic_rummy_v1());

        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::JokersNotAllowed
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
        let result = Meld::new_set(&cards, &rules());
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
        let result = Meld::new_set(&cards, &rules());
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
        let result = Meld::new_set(&cards, &rules());
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
        let result = Meld::new_run(&cards, &rules());
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
        let result = Meld::new_run(&cards, &rules());
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
        let result = Meld::new_run(&cards, &rules());
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
        let result = Meld::new_run(&cards, &rules());
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
        let result = Meld::new_run(&cards, &rules());
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
        let result = Meld::new_run(&cards, &rules());
        assert!(matches!(
            result,
            Err(GameError::MeldError(
                crate::errors::MeldError::RankHasTooManyAces
            ))
        ));
    }

    #[test]
    fn new_run_rejects_king_ace_two_wraparound() {
        let cards = vec![
            card(Rank::King, Suit::Clubs),
            card(Rank::Ace, Suit::Clubs),
            card(Rank::Two, Suit::Clubs),
        ];

        let result = Meld::new_run(&cards, &MeldRulesConfig::basic_rummy_v1());

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
        let result = Meld::new_run(&cards, &rules());
        assert!(result.is_ok());
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

        let result = Meld::new_run(&cards, &rules());

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
            let Ok(meld) = Meld::new_set(&cards, &rules()) else {
                panic!("the set should be valid");
            };
            assert_eq!(meld.card_ids, expected_ids);
        }
    }

    #[test]
    fn new_run_accepts_jokers_filling_both_ace_positions() {
        let mut cards = vec![
            card(Rank::Seven, Suit::Clubs),
            card(Rank::Eight, Suit::Clubs),
            card(Rank::Nine, Suit::Clubs),
        ];
        cards.extend((0..11).map(|_| joker()));

        let result = Meld::new_run(&cards, &rules());

        assert!(result.is_ok());
    }
}
