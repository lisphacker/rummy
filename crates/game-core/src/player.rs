use std::collections::HashMap;

use crate::{
    card::Card,
    id::{CardId, PlayerId},
    meld::Meld,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub hand: HashMap<CardId, Card>,
    pub uncategorized: Vec<CardId>,
    pub melds: Vec<Meld>,
}
