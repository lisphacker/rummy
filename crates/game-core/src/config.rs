//! Explicit configuration for variant-dependent rules.

use crate::errors::{ConfigError, GameError, GameResult};

/// Identifies the immutable rules profile used to interpret a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RulesProfile {
    /// The first, versioned Basic Rummy profile.
    BasicRummyV1,
}

/// Determines how an ace may be used in a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AcePolicy {
    /// An ace may be low or high, but runs may not wrap and an ace occupies one end.
    LowOrHighNoWrap,
}

/// Determines which cards may be taken from the discard pile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DiscardPickupRule {
    /// Only the top card may be taken.
    TopOnly,
}

/// Determines how a player completes their hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeclarationRule {
    /// Discard once and partition every remaining card into melds after drawing.
    AtomicCompleteHandWithDiscard,
}

/// Determines whether melds are exposed during normal play.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MeldVisibilityRule {
    /// Candidate melds remain private; table melds and laying off are disabled.
    PrivateDrafts,
}

/// Determines how a round is resolved after its final permitted stock recycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BlockedRoundRule {
    /// Blocked-round scoring is unsupported until the product rule is selected.
    PendingProductDecision,
}

/// Point values and the score required to complete a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScoringRules {
    ace: u8,
    numbered_cards: NumberedCardScoring,
    face_card: u8,
    round_award: RoundAwardRule,
    match_target: u32,
}

/// Determines how unmatched numbered cards are valued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NumberedCardScoring {
    /// A numbered card is worth the number printed on it.
    FaceValue,
}

/// Determines which player receives points after declarations are scored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RoundAwardRule {
    /// The completing player receives all opponents' unmatched-card points.
    DeclarerReceivesOpponentsUnmatchedTotal,
}

impl ScoringRules {
    /// Returns an ace's unmatched-card value.
    #[must_use]
    pub const fn ace(self) -> u8 {
        self.ace
    }

    /// Returns the unmatched numbered-card scoring rule.
    #[must_use]
    pub const fn numbered_cards(self) -> NumberedCardScoring {
        self.numbered_cards
    }

    /// Returns an unmatched jack, queen, or king's value.
    #[must_use]
    pub const fn face_card(self) -> u8 {
        self.face_card
    }

    /// Returns the rule used to award a completed round's points.
    #[must_use]
    pub const fn round_award(self) -> RoundAwardRule {
        self.round_award
    }

    /// Returns the score at which the match ends.
    #[must_use]
    pub const fn match_target(self) -> u32 {
        self.match_target
    }
}

/// Complete, validated rules used for one game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConfig {
    profile: RulesProfile,
    player_count: u8,
    deck_count: u8,
    jokers_per_deck: u8,
    cards_per_player: u8,
    minimum_meld_size: usize,
    ace_policy: AcePolicy,
    discard_pickup: DiscardPickupRule,
    allow_immediate_discard_pickup_rediscard: bool,
    declaration: DeclarationRule,
    meld_visibility: MeldVisibilityRule,
    maximum_stock_recycles: u8,
    blocked_round: BlockedRoundRule,
    scoring: ScoringRules,
}

impl GameConfig {
    /// Constructs the canonical `BasicRummyV1` profile for `player_count`.
    pub const fn basic_rummy_v1(player_count: u8) -> GameResult<Self> {
        if player_count < 2 || player_count > 8 {
            return Err(GameError::ConfigError(ConfigError::UnsupportedPlayerCount));
        }

        Ok(Self {
            profile: RulesProfile::BasicRummyV1,
            player_count,
            deck_count: if player_count <= 3 { 1 } else { 2 },
            jokers_per_deck: 0,
            cards_per_player: 10,
            minimum_meld_size: 3,
            ace_policy: AcePolicy::LowOrHighNoWrap,
            discard_pickup: DiscardPickupRule::TopOnly,
            allow_immediate_discard_pickup_rediscard: false,
            declaration: DeclarationRule::AtomicCompleteHandWithDiscard,
            meld_visibility: MeldVisibilityRule::PrivateDrafts,
            maximum_stock_recycles: 2,
            blocked_round: BlockedRoundRule::PendingProductDecision,
            scoring: ScoringRules {
                ace: 1,
                numbered_cards: NumberedCardScoring::FaceValue,
                face_card: 10,
                round_award: RoundAwardRule::DeclarerReceivesOpponentsUnmatchedTotal,
                match_target: 100,
            },
        })
    }

    /// Returns the versioned profile identifier.
    #[must_use]
    pub const fn profile(self) -> RulesProfile {
        self.profile
    }
    /// Returns the configured number of players.
    #[must_use]
    pub const fn player_count(self) -> u8 {
        self.player_count
    }
    /// Returns the number of standard decks used.
    #[must_use]
    pub const fn deck_count(self) -> u8 {
        self.deck_count
    }
    /// Returns the number of jokers included per deck.
    #[must_use]
    pub const fn jokers_per_deck(self) -> u8 {
        self.jokers_per_deck
    }
    /// Returns the number of cards dealt to each player.
    #[must_use]
    pub const fn cards_per_player(self) -> u8 {
        self.cards_per_player
    }
    /// Returns the minimum number of cards in a complete meld.
    #[must_use]
    pub const fn minimum_meld_size(self) -> usize {
        self.minimum_meld_size
    }
    /// Returns the run ace policy.
    #[must_use]
    pub const fn ace_policy(self) -> AcePolicy {
        self.ace_policy
    }
    /// Returns the discard-pile pickup policy.
    #[must_use]
    pub const fn discard_pickup(self) -> DiscardPickupRule {
        self.discard_pickup
    }
    /// Returns whether a just-picked-up discard may be immediately discarded.
    #[must_use]
    pub const fn allow_immediate_discard_pickup_rediscard(self) -> bool {
        self.allow_immediate_discard_pickup_rediscard
    }
    /// Returns the completion declaration policy.
    #[must_use]
    pub const fn declaration(self) -> DeclarationRule {
        self.declaration
    }
    /// Returns the normal-play meld visibility policy.
    #[must_use]
    pub const fn meld_visibility(self) -> MeldVisibilityRule {
        self.meld_visibility
    }
    /// Returns the maximum number of stock recycles permitted in a round.
    #[must_use]
    pub const fn maximum_stock_recycles(self) -> u8 {
        self.maximum_stock_recycles
    }
    /// Returns the blocked-round resolution policy.
    #[must_use]
    pub const fn blocked_round(self) -> BlockedRoundRule {
        self.blocked_round
    }
    /// Returns the scoring rules.
    #[must_use]
    pub const fn scoring(self) -> ScoringRules {
        self.scoring
    }
    /// Returns whether this profile contains any jokers.
    #[must_use]
    pub const fn allow_jokers(self) -> bool {
        self.jokers_per_deck > 0
    }

    #[cfg(test)]
    pub(crate) const fn basic_rummy_v1_with_jokers_for_testing() -> Self {
        let mut config = match Self::basic_rummy_v1(2) {
            Ok(config) => config,
            Err(_) => unreachable!(),
        };
        config.jokers_per_deck = 1;
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_rules(player_count: u8) -> GameConfig {
        let Ok(config) = GameConfig::basic_rummy_v1(player_count) else {
            panic!("test player count must be supported");
        };
        config
    }

    #[test]
    fn basic_profile_selects_deck_count_from_player_count() {
        for player_count in [2, 3] {
            assert_eq!(basic_rules(player_count).deck_count(), 1);
        }
        for player_count in 4..=8 {
            assert_eq!(basic_rules(player_count).deck_count(), 2);
        }
    }

    #[test]
    fn basic_profile_rejects_unsupported_player_counts() {
        for player_count in [0, 1, 9, u8::MAX] {
            assert_eq!(
                GameConfig::basic_rummy_v1(player_count),
                Err(GameError::ConfigError(ConfigError::UnsupportedPlayerCount))
            );
        }
    }

    #[test]
    fn basic_profile_matches_canonical_rules() {
        let config = basic_rules(4);
        assert_eq!(config.profile(), RulesProfile::BasicRummyV1);
        assert_eq!(config.player_count(), 4);
        assert_eq!(config.jokers_per_deck(), 0);
        assert_eq!(config.cards_per_player(), 10);
        assert_eq!(config.minimum_meld_size(), 3);
        assert_eq!(config.ace_policy(), AcePolicy::LowOrHighNoWrap);
        assert_eq!(config.discard_pickup(), DiscardPickupRule::TopOnly);
        assert!(!config.allow_immediate_discard_pickup_rediscard());
        assert_eq!(
            config.declaration(),
            DeclarationRule::AtomicCompleteHandWithDiscard
        );
        assert_eq!(config.meld_visibility(), MeldVisibilityRule::PrivateDrafts);
        assert_eq!(config.maximum_stock_recycles(), 2);
        assert_eq!(
            config.blocked_round(),
            BlockedRoundRule::PendingProductDecision
        );
        assert_eq!(config.scoring().ace(), 1);
        assert_eq!(
            config.scoring().numbered_cards(),
            NumberedCardScoring::FaceValue
        );
        assert_eq!(config.scoring().face_card(), 10);
        assert_eq!(
            config.scoring().round_award(),
            RoundAwardRule::DeclarerReceivesOpponentsUnmatchedTotal
        );
        assert_eq!(config.scoring().match_target(), 100);
    }
}
