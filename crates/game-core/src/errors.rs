#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MeldError {
    NotEnoughCardsForMeld,
    MeldMustHaveNonJokerCards,
    SetMustHaveSameRank,
    SetMustHaveUniqueSuits,
    SetHasTooManyJokers,
    RunMustHaveSameSuit,
    RunMustHaveConsecutiveRanks,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameError {
    MeldError(MeldError),
}

pub type GameResult<T> = Result<T, GameError>;
