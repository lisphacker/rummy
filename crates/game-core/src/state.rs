use crate::card::{Card, Rank, Suit};
use crate::config::GameConfig;
use crate::errors::{GameError, GameResult, GameStateError};
use crate::id::{CardId, GameId, PlayerId};
use crate::ordered_map::OrderedMap;
use crate::player::Player;

#[derive(Debug, Eq, PartialEq)]
pub enum TurnState {
    AwaitingDraw,
    Drawn { drawn: CardId },
    Discarded { discarded: CardId },
}

#[derive(Debug, PartialEq)]
pub enum GamePhase {
    WaitingForPlayers,
    InitializingGame,
    PlayerTurn {
        player_index: usize,
        turn_state: TurnState,
    },
    RestockingDrawStack {
        player_index: usize,
    },
    GameEndingWaitingForPlayerSubmissions {
        winning_player_id: PlayerId,
    },
    GameEnded,
}

#[derive(Debug)]
pub struct GameState {
    id: GameId,
    players: OrderedMap<PlayerId, Player>,
    deck: OrderedMap<CardId, Card>,
    draw_stack: Vec<CardId>,
    discard_pile: Vec<CardId>,
    phase: GamePhase,
    config: Option<GameConfig>,
}

impl GameState {
    pub fn new(id: GameId) -> Self {
        Self {
            id,
            players: OrderedMap::new(),
            deck: OrderedMap::new(),
            draw_stack: Vec::new(),
            discard_pile: Vec::new(),
            phase: GamePhase::WaitingForPlayers,
            config: None,
        }
    }

    pub fn initialize_game_from_config(&mut self, config: GameConfig) -> GameResult<()> {
        if self.config.is_some() {
            return Err(GameError::GameStateError(GameStateError::ConfigAlreadySet));
        }
        self.config = Some(config);
        self.initialize_deck()?;
        Ok(())
    }

    pub fn initialize_deck(&mut self) -> GameResult<()> {
        let config = self.ensure_config()?;
        let mut cards = Vec::new();
        for _ in 0..config.deck_count() {
            for suit in Suit::iter() {
                for rank in Rank::iter() {
                    let card = Card::standard(CardId::new(), suit, rank);
                    cards.push(card);
                }
            }
            for _ in 0..config.jokers_per_deck() {
                cards.push(Card::joker(CardId::new()));
            }
        }
        self.deck = cards.into_iter().map(|card| (card.id, card)).collect();
        self.draw_stack = self.deck.keys().cloned().collect();
        Ok(())
    }

    pub fn shuffle_draw_stack(&mut self) -> GameResult<()> {
        if self.phase != GamePhase::InitializingGame
            && !matches!(self.phase, GamePhase::RestockingDrawStack { .. })
        {
            return Err(GameError::GameStateError(
                GameStateError::InvalidGamePhaseForShuffle,
            ));
        }
        use rand::seq::SliceRandom;
        let mut rng = rand::rng();
        self.draw_stack.shuffle(&mut rng);
        Ok(())
    }

    fn ensure_config(&self) -> GameResult<&GameConfig> {
        self.config
            .as_ref()
            .ok_or_else(|| GameError::GameStateError(GameStateError::ConfigNotSet))
    }
}
