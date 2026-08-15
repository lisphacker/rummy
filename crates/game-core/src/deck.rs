use crate::card::Card;

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new(cards: Vec<Card>) -> Self {
        Self { cards }
    }

    pub fn draw_card(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;

        let mut rng = rand::rng();
        self.cards.shuffle(&mut rng);
    }
}
