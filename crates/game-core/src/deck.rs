use std::collections::HashMap;

use crate::{card::Card, id::CardId};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Deck {
    pub cards: HashMap<CardId, Card>,
}
