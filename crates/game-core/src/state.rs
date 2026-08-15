use crate::card::{Card, Rank, Suit};
use crate::id::CardId;
use crate::player::Player;
use crate::rules::config::GameConfig;
use std::collections::HashMap;

#[derive(Debug)]
pub struct GameState {
    pub players: Vec<Player>,
    pub deck: HashMap<CardId, Card>,
    pub draw_stack: Vec<CardId>,
    pub discard_pile: Vec<CardId>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
            deck: HashMap::new(),
            draw_stack: Vec::new(),
            discard_pile: Vec::new(),
        }
    }

    pub fn initialize_deck(&mut self, game_config: GameConfig) {
        let mut cards = Vec::new();
        for _ in 0..game_config.deck_count() {
            for suit in Suit::iter() {
                for rank in Rank::iter() {
                    let card = Card::new(suit, rank);
                    cards.push(card);
                }
            }
            for _ in 0..game_config.jokers_per_deck() {
                cards.push(Card::new_joker());
            }
        }
        self.deck = cards.into_iter().map(|card| (card.id, card)).collect();
        self.draw_stack = self.deck.keys().cloned().collect();
    }

    pub fn shuffle_draw_stack(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        self.draw_stack.shuffle(&mut rng);
    }
}
