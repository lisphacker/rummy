#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MeldError {
    NotEnoughCardsForMeld,
    MeldMustHaveNonJokerCards,
    MeldHasTooManyJokerCards,
    SetMustHaveSameRank,
    SetMustHaveUniqueSuits,
    SetHasTooManyJokers,
    RunMustHaveSameSuit,
    RunMustHaveConsecutiveRanks,
    RankHasTooManyAces,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameError {
    MeldError(MeldError),
}

pub type GameResult<T> = Result<T, GameError>;
