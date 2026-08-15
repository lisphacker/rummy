use crate::card::Card;
use crate::id::CardId;
use crate::player::Player;
use std::collections::HashMap;

#[derive(Debug)]
pub struct GameState {
    pub players: Vec<Player>,
    pub deck: HashMap<CardId, Card>,
    pub draw_stock: Vec<CardId>,
    pub discard_pile: Vec<CardId>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            deck: HashMap::new(),
            draw_stock: Vec::new(),
            discard_pile: Vec::new(),
        }
    }
}
