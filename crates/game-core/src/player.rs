use std::collections::HashMap;

use crate::{
    card::Card,
    id::{CardId, PlayerId},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub hand: HashMap<CardId, Card>,
}
