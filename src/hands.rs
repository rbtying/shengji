use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::{Card, EffectiveSuit, PlayerID, Trump};

#[derive(Error, Clone, Debug, Serialize, Deserialize)]
pub enum HandError {
    #[error("unknown player ID {:?}", _0)]
    UnknownPlayerID(PlayerID),
    #[error("cards not found in hand")]
    CardsNotFound,
    #[error("cards cannot be played")]
    CardsNotPlayable,
    #[error("trump not set yet")]
    TrumpNotSet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hands {
    hands: HashMap<PlayerID, HashMap<Card, usize>>,
    trump: Option<Trump>,
}

impl Hands {
    pub fn new(players: impl IntoIterator<Item = PlayerID>) -> Self {
        Hands {
            hands: players.into_iter().map(|id| (id, HashMap::new())).collect(),
            trump: None,
        }
    }

    pub fn drop_other_players(&mut self, id: PlayerID) {
        self.hands.retain(|pid, _| *pid == id);
    }

    pub fn get(&self, id: PlayerID) -> Result<&'_ HashMap<Card, usize>, HandError> {
        self.exists(id)?;
        Ok(&self.hands[&id])
    }

    pub fn set_trump(&mut self, trump: Trump) {
        self.trump = Some(trump);
    }

    pub fn trump(&self) -> Result<Trump, HandError> {
        match self.trump {
            Some(trump) => Ok(trump),
            None => Err(HandError::TrumpNotSet),
        }
    }

    pub fn exists(&self, id: PlayerID) -> Result<(), HandError> {
        if self.hands.contains_key(&id) {
            Ok(())
        } else {
            Err(HandError::UnknownPlayerID(id))
        }
    }

    pub fn contains(
        &self,
        id: PlayerID,
        cards: impl IntoIterator<Item = Card>,
    ) -> Result<(), HandError> {
        self.exists(id)?;

        let required = Card::count(cards);

        for (card, number) in required {
            if self.hands[&id].get(&card).cloned().unwrap_or(0) < number {
                return Err(HandError::CardsNotFound);
            }
        }
        Ok(())
    }

    pub fn is_void(&self, id: PlayerID, suit: EffectiveSuit) -> Result<bool, HandError> {
        self.exists(id)?;
        let trump = self.trump()?;

        for (card, number) in &self.hands[&id] {
            if *number > 0 && trump.effective_suit(*card) == suit {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub fn counts(&self, id: PlayerID) -> Option<&'_ HashMap<Card, usize>> {
        self.hands.get(&id)
    }

    pub fn is_empty(&self) -> bool {
        !self.hands.values().any(|h| h.values().any(|c| *c > 0))
    }

    pub fn cards(&self, id: PlayerID) -> Result<Vec<Card>, HandError> {
        self.exists(id)?;
        let mut cards = vec![];
        for (card, number) in &self.hands[&id] {
            for _ in 0..*number {
                cards.push(*card);
            }
        }
        if let Some(trump) = self.trump {
            cards.sort_by(|a, b| trump.compare(*a, *b));
        } else {
            cards.sort_by_key(|c| c.as_char());
        }
        Ok(cards)
    }

    pub fn add(
        &mut self,
        id: PlayerID,
        cards: impl IntoIterator<Item = Card>,
    ) -> Result<(), HandError> {
        self.exists(id)?;
        let hand = self.hands.get_mut(&id).unwrap();
        for card in cards {
            *hand.entry(card).or_insert(0) += 1;
        }
        Ok(())
    }

    pub fn remove(
        &mut self,
        id: PlayerID,
        cards: impl IntoIterator<Item = Card> + Clone,
    ) -> Result<(), HandError> {
        self.contains(id, cards.clone())?;

        let hand = self.hands.get_mut(&id).unwrap();
        for card in cards {
            *hand.entry(card).or_insert(0) -= 1;
        }
        Ok(())
    }
}
