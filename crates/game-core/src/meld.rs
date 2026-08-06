use crate::id::CardId;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MeldType {
    Set,
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Meld {
    pub meld_type: MeldType,
    pub cards: Vec<CardId>,
}
