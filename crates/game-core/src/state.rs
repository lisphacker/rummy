use crate::card::{Card, Rank, Suit};
use crate::config::GameConfig;
use crate::id::{CardId, GameId, PlayerId};
use crate::ordered_map::OrderedMap;
use crate::player::Player;

#[derive(Debug)]
pub enum GamePhase {
    WaitingForPlayers,
    PlayerTurn { player_index: usize },
    GameEndingWaitingForPlayerSubmissions,
    GameEnded,
}

#[derive(Debug)]
pub struct GameState {
    pub id: GameId,
    pub players: OrderedMap<PlayerId, Player>,
    pub deck: OrderedMap<CardId, Card>,
    pub draw_stack: Vec<CardId>,
    pub discard_pile: Vec<CardId>,
    pub phase: GamePhase,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            id: GameId::new(),
            players: OrderedMap::new(),
            deck: OrderedMap::new(),
            draw_stack: Vec::new(),
            discard_pile: Vec::new(),
            phase: GamePhase::WaitingForPlayers,
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
