use std::collections::HashSet;
use std::ops::{Deref, DerefMut};

use anyhow::{anyhow, bail, Error};
use rand::{seq::SliceRandom, RngCore};
use serde::{Deserialize, Serialize};

use crate::hands::Hands;
use crate::settings::{GameMode, GameModeSettings, GameStartPolicy, PropagatedState};
use crate::types::{Card, Number, PlayerID, Rank, ALL_SUITS};

use crate::game_state::DrawPhase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializePhase {
    propagated: PropagatedState,
}

impl InitializePhase {
    pub fn new() -> Self {
        Self {
            propagated: PropagatedState::default(),
        }
    }

    pub fn from_propagated(propagated: PropagatedState) -> Self {
        Self { propagated }
    }

    pub fn propagated(&self) -> &PropagatedState {
        &self.propagated
    }

    pub fn propagated_mut(&mut self) -> &mut PropagatedState {
        &mut self.propagated
    }

    pub fn start(&self, id: PlayerID) -> Result<DrawPhase, Error> {
        if self.propagated.players.len() < 4 {
            bail!("not enough players")
        }

        if self.propagated.game_start_policy == GameStartPolicy::AllowLandlordOnly
            && self.propagated.landlord.map(|l| l != id).unwrap_or(false)
        {
            bail!("Only the landlord can start the game")
        }

        let game_mode = match self.propagated.game_mode {
            GameModeSettings::FindingFriends {
                num_friends: Some(num_friends),
                ..
            } if num_friends + 1 < self.propagated.players.len() => GameMode::FindingFriends {
                num_friends,
                friends: vec![],
            },
            GameModeSettings::FindingFriends { .. } => GameMode::FindingFriends {
                num_friends: (self.propagated.players.len() / 2) - 1,
                friends: vec![],
            },
            GameModeSettings::Tractor if self.propagated.players.len() % 2 == 0 => {
                GameMode::Tractor
            }
            GameModeSettings::Tractor => {
                bail!("can only play tractor with an even number of players")
            }
        };

        let mut rng = rand::thread_rng();

        let position = self
            .propagated
            .landlord
            .and_then(|landlord| {
                self.propagated
                    .players
                    .iter()
                    .position(|p| p.id == landlord)
            })
            .unwrap_or(rng.next_u32() as usize % self.propagated.players.len());

        let level = if self.propagated.landlord.is_some() {
            Some(self.propagated.players[position].rank())
        } else {
            None
        };

        let num_decks = self.propagated.num_decks();
        if num_decks == 0 {
            bail!("need at least one deck to start the game");
        }
        let decks = self.propagated.decks()?;
        let mut deck = Vec::with_capacity(decks.iter().map(|d| d.len()).sum::<usize>());
        for deck_ in &decks {
            deck.extend(deck_.cards());
        }
        // Ensure that it is possible to bid for the landlord, if set, or all players, if not.
        match level {
            Some(Rank::Number(level)) if decks.iter().any(|d| d.includes_number(level)) => (),
            Some(Rank::NoTrump) => (),
            None if self.players.iter().all(|p| {
                decks.iter().any(|d| match p.level {
                    Rank::Number(level) => d.includes_number(level),
                    Rank::NoTrump => true,
                })
            }) => {}
            _ => bail!("deck configuration is missing cards needed to bid"),
        }

        deck.shuffle(&mut rng);

        let mut removed_cards = vec![];

        let kitty_size = match self.propagated.kitty_size {
            Some(size)
                if deck.len() % self.propagated.players.len()
                    == size % self.propagated.players.len() =>
            {
                size
            }
            Some(size) => {
                // Remove cards from the deck, until the deck and kitty together work out to the
                // appropriate number of cards.
                let num_players = self.propagated.players.len();

                let min_number: Number = decks
                    .iter()
                    .map(|d| d.min)
                    .min()
                    .ok_or_else(|| anyhow!("no minimum value in deck?"))?;

                // Choose a card to remove that doesn't unfairly disadvantage a particular player,
                // and ideally isn't points either.
                let removed_card_number = match level {
                    Some(Rank::Number(level)) if level == min_number => {
                        // If the minimum value isn't an A, this will be reasonable, otherwise
                        // it'll remove a trump card from the deck...
                        min_number.successor().unwrap_or(min_number)
                    }
                    Some(_) => min_number,
                    None => {
                        let mut bad_levels = self
                            .propagated
                            .players
                            .iter()
                            .flat_map(|p| match p.level {
                                Rank::Number(n) => Some(n),
                                Rank::NoTrump => None,
                            })
                            .collect::<HashSet<Number>>();
                        bad_levels.insert(Number::Five);
                        bad_levels.insert(Number::Ten);
                        bad_levels.insert(Number::King);
                        let mut n = min_number;
                        loop {
                            if !bad_levels.contains(&n) {
                                break n;
                            }
                            n = match n.successor() {
                                Some(nn) => nn,
                                // If we somehow have enough players that we can't remove cards
                                // without disadvantaging _someone_, or choosing points,
                                // arbitrarily choose to remove twos.
                                None => break min_number,
                            };
                        }
                    }
                };

                let mut suit_idx = ALL_SUITS.len() - 1;

                while deck.len() % num_players != size % num_players {
                    let card_to_remove = Card::Suited {
                        suit: ALL_SUITS[suit_idx],
                        number: removed_card_number,
                    };
                    suit_idx = if suit_idx == 0 {
                        ALL_SUITS.len() - 1
                    } else {
                        suit_idx - 1
                    };

                    // Attempt to remove the card from the deck.
                    match deck.iter().position(|c| *c == card_to_remove) {
                        Some(idx) => {
                            deck.remove(idx);
                            removed_cards.push(card_to_remove);
                        }
                        // Note: we would only hit this case if there are fewer decks than players,
                        // which should be prevented in the settings layer.
                        None => bail!(format!(
                            "Couldn't find {:?} in the deck to remove",
                            card_to_remove
                        )),
                    }
                }
                size
            }
            None => {
                let mut kitty_size = deck.len() % self.propagated.players.len();
                if kitty_size == 0 {
                    kitty_size = self.propagated.players.len();
                }
                if kitty_size < 5 {
                    kitty_size += self.propagated.players.len();
                }
                kitty_size
            }
        };

        let propagated = self.propagated.clone();

        Ok(DrawPhase {
            deck: (&deck[0..deck.len() - kitty_size]).to_vec(),
            kitty: (&deck[deck.len() - kitty_size..]).to_vec(),
            hands: Hands::new(self.propagated.players.iter().map(|p| p.id)),
            bids: Vec::new(),
            revealed_cards: 0,
            autobid: None,
            propagated,
            position,
            num_decks,
            decks,
            game_mode,
            level,
            removed_cards,
        })
    }
}

impl Deref for InitializePhase {
    type Target = PropagatedState;

    fn deref(&self) -> &PropagatedState {
        &self.propagated
    }
}

impl DerefMut for InitializePhase {
    fn deref_mut(&mut self) -> &mut PropagatedState {
        &mut self.propagated
    }
}
