#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MeldError {
    NotEnoughCardsForMeld,
    MeldMustHaveNonJokerCards,
    MeldHasTooManyJokerCards,
    JokersNotAllowed,
    SetMustHaveSameRank,
    SetMustHaveUniqueSuits,
    SetHasTooManyJokers,
    RunMustHaveSameSuit,
    RunMustHaveConsecutiveRanks,
    RankHasTooManyAces,
    DuplicateCardsInMeld,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HandError {
    CannotMoveCard,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConfigError {
    UnsupportedPlayerCount,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameStateError {
    ConfigNotSet,
    ConfigAlreadySet,
    InvalidGamePhaseForShuffle,
    NoPlayers,
    CardNotFoundInDeck,
    NotEnoughCardsInDrawStack,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameError {
    MeldError(MeldError),
    HandError(HandError),
    ConfigError(ConfigError),
    GameStateError(GameStateError),
}

pub type GameResult<T> = Result<T, GameError>;
