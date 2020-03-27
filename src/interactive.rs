use std::sync::{Arc, Mutex};

use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};

use crate::game_state::{GameState, InitializePhase};
use crate::types::{Card, PlayerID};

#[derive(Clone, Debug)]
pub struct InteractiveGame {
    state: Arc<Mutex<GameState>>,
}

impl InteractiveGame {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(GameState::Initialize(InitializePhase::new()))),
        }
    }

    pub fn register(&self, name: String) -> Result<PlayerID, Error> {
        let mut s = self.state.lock().unwrap();
        if let Some(players) = s.players() {
            for p in players {
                if p.name == name {
                    return Ok(p.id);
                }
            }
        }
        if let GameState::Initialize(ref mut phase) = *s {
            Ok(phase.add_player(name))
        } else {
            bail!("game already started")
        }
    }

    pub fn kick(&self, id: PlayerID) -> Result<(), Error> {
        let mut s = self.state.lock().unwrap();
        if let GameState::Initialize(ref mut phase) = *s {
            Ok(phase.remove_player(id))
        } else {
            bail!("game already started")
        }
    }

    pub fn dump_state_for_player(&self, id: PlayerID) -> GameState {
        self.state.lock().unwrap().for_player(id)
    }

    pub fn interact(&self, msg: Message, id: PlayerID) -> Result<(), Error> {
        let mut s = self.state.lock().unwrap();

        match (msg, &mut *s) {
            (Message::EndGame, _) => {
                *s = GameState::Done;
                Ok(())
            }
            (Message::StartGame, GameState::Initialize(ref mut state)) => {
                *s = GameState::Draw(state.start()?);
                Ok(())
            }
            (Message::SetNumDecks(num_decks), GameState::Initialize(ref mut state)) => {
                state.set_num_decks(num_decks);
                Ok(())
            }
            (Message::DrawCard, GameState::Draw(ref mut state)) => {
                state.draw_card(id)?;
                Ok(())
            }
            (Message::Bid(card, count), GameState::Draw(ref mut state)) => {
                if state.bid(id, card, count) {
                    Ok(())
                } else {
                    bail!("bid was invalid")
                }
            }
            (Message::PickUpKitty, GameState::Draw(ref mut state)) => {
                state.advance(id)?;
                Ok(())
            }
            (Message::MoveCardToKitty(card), GameState::Exchange(ref mut state)) => {
                state.move_card_to_kitty(id, card)?;
                Ok(())
            }
            (Message::MoveCardToHand(card), GameState::Exchange(ref mut state)) => {
                state.move_card_to_hand(id, card)?;
                Ok(())
            }
            (Message::BeginPlay, GameState::Exchange(ref mut state)) => {
                *s = GameState::Play(state.advance(id)?);
                Ok(())
            }
            (Message::PlayCards(ref cards), GameState::Play(ref mut state)) => {
                state.play_cards(id, cards)?;
                Ok(())
            }
            (Message::TakeBackCards, GameState::Play(ref mut state)) => {
                state.take_back_cards(id)?;
                Ok(())
            }
            (Message::StartNewGame, GameState::Play(ref mut state)) => {
                *s = GameState::Initialize(state.finish_game()?);
                Ok(())
            }
            _ => bail!("not supported in current phase"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    EndGame,
    SetNumDecks(usize),
    StartGame,
    DrawCard,
    Bid(Card, usize),
    PickUpKitty,
    MoveCardToKitty(Card),
    MoveCardToHand(Card),
    BeginPlay,
    PlayCards(Vec<Card>),
    TakeBackCards,
    StartNewGame,
}
