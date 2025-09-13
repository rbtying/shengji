use std::collections::hash_map::Entry;
use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::types::{Card, EffectiveSuit, PlayerID, Trump};

#[derive(Error, Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum HandError {
    #[error("unknown player ID {:?}", _0)]
    UnknownPlayerID(PlayerID),
    #[error("cards not found in hand")]
    CardsNotFound,
    #[error("cards cannot be played")]
    CardsNotPlayable,
    #[error("unknown cards can't be added to hand")]
    CardNotKnown,
    #[error("trump not set yet")]
    TrumpNotSet,
}

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct Hands {
    hands: HashMap<PlayerID, HashMap<Card, usize>>,
    trump: Option<Trump>,
}

// Custom Deserialize implementation to handle string keys for PlayerID
impl<'de> Deserialize<'de> for Hands {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use crate::types::PlayerID;
        use serde::de;
        use std::str::FromStr;

        #[derive(Deserialize)]
        struct HandsHelper {
            hands: serde_json::Value,
            trump: Option<Trump>,
        }

        let helper = HandsHelper::deserialize(deserializer)?;

        // Parse the hands field manually
        let mut hands_map = HashMap::new();
        if let serde_json::Value::Object(obj) = helper.hands {
            for (key, value) in obj {
                let player_id = PlayerID::from_str(&key)
                    .map_err(|_| de::Error::custom(format!("invalid PlayerID: {}", key)))?;
                let card_map: HashMap<Card, usize> =
                    serde_json::from_value(value).map_err(de::Error::custom)?;
                hands_map.insert(player_id, card_map);
            }
        } else {
            return Err(de::Error::custom("hands must be an object"));
        }

        Ok(Hands {
            hands: hands_map,
            trump: helper.trump,
        })
    }
}

impl Hands {
    pub fn new(players: impl IntoIterator<Item = PlayerID>) -> Self {
        Hands {
            hands: players.into_iter().map(|id| (id, HashMap::new())).collect(),
            trump: None,
        }
    }

    pub fn destructively_redact_except_for_player(&mut self, id: PlayerID) {
        for (pid, cards) in &mut self.hands {
            if *pid != id {
                let count = cards.values().sum();
                cards.clear();
                cards.insert(Card::Unknown, count);
            }
        }
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

    pub fn _get_cards(&self, id: PlayerID) -> Result<Vec<Card>, HandError> {
        self.exists(id)?;
        let mut cards = Card::cards(self.hands[&id].iter())
            .copied()
            .collect::<Vec<Card>>();
        if let Some(trump) = self.trump {
            cards.sort_by(|a, b| trump.compare(*a, *b));
        } else {
            cards.sort_by(|a, b| Trump::NoTrump { number: None }.compare(*a, *b));
        }
        Ok(cards)
    }

    pub fn add(
        &mut self,
        id: PlayerID,
        cards: impl IntoIterator<Item = Card> + Clone,
    ) -> Result<(), HandError> {
        self.exists(id)?;
        let hand = self.hands.get_mut(&id).unwrap();
        for card in cards.clone() {
            if let Card::Unknown = card {
                return Err(HandError::CardNotKnown);
            }
        }
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
            if let Entry::Occupied(mut o) = hand.entry(card) {
                *o.get_mut() -= 1;
                if *o.get() == 0 {
                    o.remove();
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Hands;
    use crate::types::{
        cards::{S_2, S_3, S_4, S_5},
        PlayerID,
    };

    const P1: PlayerID = PlayerID(1);
    const P2: PlayerID = PlayerID(2);
    const P3: PlayerID = PlayerID(3);
    const P4: PlayerID = PlayerID(4);

    #[test]
    fn test_add_remove() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4]);
        hands.add(P1, vec![S_2, S_3, S_5]).unwrap();
        hands.add(P2, vec![S_2, S_3, S_5]).unwrap();
        hands.add(P3, vec![S_2, S_3, S_5]).unwrap();
        hands.add(P4, vec![S_2, S_3, S_5]).unwrap();

        hands.remove(P1, Some(S_2)).unwrap();
        hands.remove(P1, Some(S_3)).unwrap();
        hands.remove(P1, Some(S_5)).unwrap();
        hands.remove(P1, Some(S_5)).unwrap_err();
        assert!(hands._get_cards(P1).unwrap().is_empty());

        hands.remove(P2, vec![S_2, S_3, S_5]).unwrap();
        assert!(hands._get_cards(P2).unwrap().is_empty());

        hands.remove(P3, vec![S_2, S_3, S_4, S_5]).unwrap_err();
        assert_eq!(hands._get_cards(P3).unwrap(), hands._get_cards(P4).unwrap());
    }
}
