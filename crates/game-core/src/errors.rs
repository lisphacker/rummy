#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GameError {}

pub type GameResult<T> = Result<T, GameError>;
