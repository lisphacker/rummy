//! Explicit configuration for variant-dependent rules.

/// Rules that affect meld construction and extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GameConfig {
    minimum_meld_size: usize,
    allow_jokers: bool,
}

impl GameConfig {
    /// Returns the meld rules for the canonical `BasicRummyV1` profile.
    #[must_use]
    pub const fn basic_rummy_v1() -> Self {
        Self {
            minimum_meld_size: 3,
            allow_jokers: false,
        }
    }

    /// Creates meld rules for a profile.
    ///
    /// `minimum_meld_size` is clamped to one so a configuration cannot make an
    /// empty meld valid.
    #[must_use]
    pub const fn new(minimum_meld_size: usize, allow_jokers: bool) -> Self {
        Self {
            minimum_meld_size: if minimum_meld_size == 0 {
                1
            } else {
                minimum_meld_size
            },
            allow_jokers,
        }
    }

    /// Returns the minimum number of cards in a complete meld.
    #[must_use]
    pub const fn minimum_meld_size(self) -> usize {
        self.minimum_meld_size
    }

    /// Returns whether melds may contain jokers.
    #[must_use]
    pub const fn allow_jokers(self) -> bool {
        self.allow_jokers
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self::basic_rummy_v1()
    }
}
