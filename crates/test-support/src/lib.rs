//! Deterministic, framework-independent test fixtures for the Rummy workspace.
//!
//! These helpers deliberately avoid random UUIDs, wall-clock time, and
//! production randomness. Tests can therefore describe a setup with small
//! sequence numbers and reproduce it exactly on every run.

use game_core::{
    card::{Card, Rank, Suit},
    id::{CardId, GameId, PlayerId},
    player::Player,
};
use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;

const CARD_ID_NAMESPACE: u128 = 0xca4d_0000_0000_0000_0000_0000_0000_0000;
const PLAYER_ID_NAMESPACE: u128 = 0xb1a4_0000_0000_0000_0000_0000_0000_0000;
const GAME_ID_NAMESPACE: u128 = 0x6a4e_0000_0000_0000_0000_0000_0000_0000;

/// Returns a stable card ID for a test-local sequence number.
#[must_use]
pub fn card_id(sequence: u64) -> CardId {
    CardId(uuid::Uuid::from_u128(
        CARD_ID_NAMESPACE | u128::from(sequence),
    ))
}

/// Returns a stable player ID for a test-local sequence number.
#[must_use]
pub fn player_id(sequence: u64) -> PlayerId {
    PlayerId(uuid::Uuid::from_u128(
        PLAYER_ID_NAMESPACE | u128::from(sequence),
    ))
}

/// Returns a stable game ID for a test-local sequence number.
#[must_use]
pub fn game_id(sequence: u64) -> GameId {
    GameId(uuid::Uuid::from_u128(
        GAME_ID_NAMESPACE | u128::from(sequence),
    ))
}

/// Builds a standard card with a stable ID.
#[must_use]
pub fn standard_card(sequence: u64, suit: Suit, rank: Rank) -> Card {
    Card::standard(card_id(sequence), suit, rank)
}

/// Builds standard 52-card decks in deck, suit, then rank order.
///
/// Each physical card receives a distinct stable ID, including identical card
/// faces belonging to different decks. Jokers are intentionally omitted to
/// match `BasicRummyV1`.
#[must_use]
pub fn ordered_standard_decks(deck_count: usize) -> Vec<Card> {
    let mut next_sequence = 0_u64;
    let mut cards = Vec::with_capacity(deck_count.saturating_mul(52));

    for _ in 0..deck_count {
        for suit in Suit::iter() {
            for rank in Rank::iter() {
                cards.push(standard_card(next_sequence, suit, rank));
                next_sequence = next_sequence.saturating_add(1);
            }
        }
    }

    cards
}

/// Creates players with stable identities and optional predefined hands.
#[derive(Debug, Clone)]
pub struct PlayerBuilder {
    id: PlayerId,
    hand: Vec<Card>,
}

impl PlayerBuilder {
    /// Starts a player fixture using a stable sequence-based ID.
    #[must_use]
    pub fn new(sequence: u64) -> Self {
        Self {
            id: player_id(sequence),
            hand: Vec::new(),
        }
    }

    /// Adds one card to the player's initial hand.
    #[must_use]
    pub fn with_card(mut self, card: Card) -> Self {
        self.hand.push(card);
        self
    }

    /// Adds cards to the player's initial hand in iteration order.
    #[must_use]
    pub fn with_cards(mut self, cards: impl IntoIterator<Item = Card>) -> Self {
        self.hand.extend(cards);
        self
    }

    /// Builds the player, preserving hand order in `uncategorized_cards`.
    #[must_use]
    pub fn build(self) -> Player {
        let mut player = Player::new(self.id);
        for card in self.hand {
            player.draw_card(card);
        }
        player
    }
}

/// Shuffles a slice reproducibly using the supplied seed.
pub fn shuffle_with_seed<T>(values: &mut [T], seed: u64) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    values.shuffle(&mut rng);
}

/// Returns a reproducibly shuffled copy of a slice.
#[must_use]
pub fn shuffled_with_seed<T: Clone>(values: &[T], seed: u64) -> Vec<T> {
    let mut shuffled = values.to_vec();
    shuffle_with_seed(&mut shuffled, seed);
    shuffled
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use game_core::card::CardFace;

    #[test]
    fn fixture_ids_are_stable_and_namespaced() {
        assert_eq!(card_id(7), card_id(7));
        assert_eq!(player_id(7), player_id(7));
        assert_ne!(card_id(7).0, player_id(7).0);
    }

    #[test]
    fn ordered_decks_have_unique_physical_cards() {
        let cards = ordered_standard_decks(2);
        let unique_ids: HashSet<_> = cards.iter().map(|card| card.id).collect();

        assert_eq!(cards.len(), 104);
        assert_eq!(unique_ids.len(), 104);
        assert_eq!(
            cards.first().map(|card| card.face),
            Some(CardFace::Standard {
                rank: Rank::Ace,
                suit: Suit::Clubs,
            })
        );
        assert_eq!(
            cards.get(52).map(|card| card.face),
            cards.first().map(|card| card.face)
        );
        assert_ne!(
            cards.get(52).map(|card| card.id),
            cards.first().map(|card| card.id)
        );
    }

    #[test]
    fn player_builder_preserves_supplied_hand_order() {
        let cards = [
            standard_card(1, Suit::Hearts, Rank::Seven),
            standard_card(2, Suit::Spades, Rank::King),
        ];
        let player = PlayerBuilder::new(3).with_cards(cards).build();

        assert_eq!(player.id, player_id(3));
        assert_eq!(player.uncategorized_cards, cards.map(|card| card.id));
        assert_eq!(player.hand.len(), 2);
    }

    #[test]
    fn seeded_shuffle_is_repeatable_and_seed_sensitive() {
        let values: Vec<_> = (0..32).collect();

        assert_eq!(
            shuffled_with_seed(&values, 42),
            shuffled_with_seed(&values, 42)
        );
        assert_ne!(
            shuffled_with_seed(&values, 42),
            shuffled_with_seed(&values, 43)
        );
    }
}
