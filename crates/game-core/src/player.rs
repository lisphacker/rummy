use std::collections::HashMap;

use crate::{
    card::Card,
    errors::{self, GameResult},
    id::{CardId, PlayerId},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub hand: HashMap<CardId, Card>,
    pub draft_melds: Vec<Vec<CardId>>,
    pub uncategorized_cards: Vec<CardId>,
}

impl Player {
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            hand: HashMap::new(),
            draft_melds: Vec::new(),
            uncategorized_cards: Vec::new(),
        }
    }

    pub fn dealt(mut self, cards: impl IntoIterator<Item = Card>) -> Self {
        for card in cards {
            self.draw_card(card);
        }
        self
    }

    pub fn draw_card(&mut self, card: Card) {
        self.hand.insert(card.id, card);
        self.uncategorized_cards.push(card.id);
    }

    pub fn discard_card(&mut self, card_id: CardId) -> Option<Card> {
        if let Some(card) = self.hand.remove(&card_id) {
            self.uncategorized_cards.retain(|&id| id != card_id);
            for meld in &mut self.draft_melds {
                meld.retain(|&id| id != card_id);
            }
            Some(card)
        } else {
            None
        }
    }

    pub fn move_card_between_melds(
        &mut self,
        card_id: CardId,
        from_meld_index: usize,
        to_meld_index: usize,
    ) -> GameResult<()> {
        if from_meld_index >= self.draft_melds.len() {
            return Err(errors::GameError::HandError(
                errors::HandError::CannotMoveCard,
            ));
        }

        let mut to_meld_index = to_meld_index;
        if to_meld_index >= self.draft_melds.len() {
            self.draft_melds.push(Vec::new());
            to_meld_index = self.draft_melds.len() - 1;
        }

        let from_meld = &mut self.draft_melds[from_meld_index];
        if let Some(pos) = from_meld.iter().position(|&id| id == card_id) {
            from_meld.remove(pos);
            self.draft_melds[to_meld_index].push(card_id);
            Ok(())
        } else {
            Err(errors::GameError::HandError(
                errors::HandError::CannotMoveCard,
            ))
        }
    }

    pub fn move_card_to_uncategorized(
        &mut self,
        card_id: CardId,
        from_meld_index: usize,
    ) -> GameResult<()> {
        if from_meld_index >= self.draft_melds.len() {
            return Err(errors::GameError::HandError(
                errors::HandError::CannotMoveCard,
            ));
        }

        let from_meld = &mut self.draft_melds[from_meld_index];
        if let Some(pos) = from_meld.iter().position(|&id| id == card_id) {
            from_meld.remove(pos);
            self.uncategorized_cards.push(card_id);
            Ok(())
        } else {
            Err(errors::GameError::HandError(
                errors::HandError::CannotMoveCard,
            ))
        }
    }

    pub fn move_card_from_uncategorized_to_meld(
        &mut self,
        card_id: CardId,
        to_meld_index: usize,
    ) -> GameResult<()> {
        if let Some(pos) = self
            .uncategorized_cards
            .iter()
            .position(|&id| id == card_id)
        {
            self.uncategorized_cards.remove(pos);
            let mut to_meld_index = to_meld_index;
            if to_meld_index >= self.draft_melds.len() {
                self.draft_melds.push(Vec::new());
                to_meld_index = self.draft_melds.len() - 1;
            }
            self.draft_melds[to_meld_index].push(card_id);
            Ok(())
        } else {
            Err(errors::GameError::HandError(
                errors::HandError::CannotMoveCard,
            ))
        }
    }
}
