use serde::{Deserialize, Serialize};
use slog_derive::KV;

use crate::types::{Card, Number, FULL_DECK};

#[derive(Clone, Debug, Serialize, Deserialize, KV)]
pub struct Deck {
    pub exclude_small_joker: bool,
    pub exclude_big_joker: bool,
    pub min: Number,
}

impl slog::Value for Deck {
    fn serialize(
        &self,
        _: &slog::Record,
        key: slog::Key,
        serializer: &mut dyn slog::Serializer,
    ) -> slog::Result {
        serializer.emit_str(key, &format!("{:?}", self))
    }
}

impl Default for Deck {
    fn default() -> Self {
        Deck {
            exclude_small_joker: false,
            exclude_big_joker: false,
            min: Number::Two,
        }
    }
}

impl Deck {
    pub fn includes_number(&self, number: Number) -> bool {
        number >= self.min
    }

    pub fn includes_card(&self, card: Card) -> bool {
        match card {
            Card::BigJoker if self.exclude_big_joker => false,
            Card::SmallJoker if self.exclude_small_joker => false,
            Card::Suited { number, .. } if !self.includes_number(number) => false,
            _ => true,
        }
    }

    pub fn points(&self) -> usize {
        let mut pts = 0;
        if self.includes_number(Number::Five) {
            pts += 5 * 4;
        }
        if self.includes_number(Number::Ten) {
            pts += 10 * 4;
        }
        if self.includes_number(Number::King) {
            pts += 10 * 4;
        }
        pts
    }

    pub fn len(&self) -> usize {
        let mut cards = 54;
        if self.exclude_big_joker {
            cards -= 1;
        }
        if self.exclude_small_joker {
            cards -= 1;
        }

        let mut n = Number::Two;
        while n < self.min {
            cards -= 4;
            if let Some(nn) = n.successor() {
                n = nn;
            } else {
                break;
            }
        }

        cards
    }

    pub fn cards(&'_ self) -> impl Iterator<Item = Card> + '_ {
        DeckIterator {
            deck: self,
            index: 0,
        }
    }
}

pub struct DeckIterator<'d> {
    deck: &'d Deck,
    index: usize,
}

impl<'d> Iterator for DeckIterator<'d> {
    type Item = Card;
    fn next(&mut self) -> Option<Card> {
        loop {
            if self.index >= FULL_DECK.len() {
                break None;
            }
            let card = FULL_DECK[self.index];
            self.index += 1;

            if self.deck.includes_card(card) {
                break Some(card);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Number;

    use super::Deck;

    #[test]
    fn test_deck_points_calc() {
        let cases = vec![
            (Deck::default(), 54, 100),
            (
                Deck {
                    exclude_big_joker: true,
                    exclude_small_joker: true,
                    ..Default::default()
                },
                52,
                100,
            ),
            (
                Deck {
                    min: Number::Five,
                    ..Default::default()
                },
                42,
                100,
            ),
            (
                Deck {
                    min: Number::Jack,
                    ..Default::default()
                },
                18,
                40,
            ),
        ];

        for (deck, cards, points) in cases {
            eprintln!("testing {:?} {:?} {:?}", deck, cards, points);
            assert_eq!(deck.points(), points);
            assert_eq!(deck.len(), cards);
            assert_eq!(deck.cards().count(), cards);
            assert_eq!(deck.cards().flat_map(|c| c.points()).sum::<usize>(), points);
        }
    }
}
