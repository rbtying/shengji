use std::sync::{Arc, Mutex};

use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};

use crate::game_state::{Friend, GameMode, GameState, InitializePhase};
use crate::types::{Card, Number, PlayerID};

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
        if let Ok(mut s) = self.state.lock() {
            s.register(name)
        } else {
            bail!("lock poisoned")
        }
    }

    pub fn kick(&self, id: PlayerID) -> Result<(), Error> {
        if let Ok(mut s) = self.state.lock() {
            s.kick(id)
        } else {
            bail!("lock poisoned")
        }
    }

    pub fn dump_state_for_player(&self, id: PlayerID) -> Result<(GameState, Vec<Card>), Error> {
        if let Ok(s) = self.state.lock() {
            Ok((s.for_player(id), s.cards(id)))
        } else {
            bail!("lock poisoned")
        }
    }

    pub fn interact(&self, msg: Message, id: PlayerID) -> Result<Vec<String>, Error> {
        if let Ok(mut s) = self.state.lock() {
            match (msg, &mut *s) {
                (Message::EndGame, _) => {
                    *s = GameState::Done;
                    Ok(vec![])
                }
                (Message::StartGame, GameState::Initialize(ref mut state)) => {
                    *s = GameState::Draw(state.start()?);
                    Ok(vec!["Starting game".to_string()])
                }
                (Message::SetNumDecks(num_decks), GameState::Initialize(ref mut state)) => {
                    state.set_num_decks(num_decks);
                    Ok(vec![])
                }
                (Message::ReorderPlayers(ref players), GameState::Initialize(ref mut state)) => {
                    state.reorder_players(&players)?;
                    Ok(vec![])
                }
                (Message::SetRank(rank), GameState::Initialize(ref mut state)) => {
                    state.set_rank(id, rank)?;
                    let n = s.player_name(id)?;
                    Ok(vec![format!("{} set their rank to {}", n, rank.as_str())])
                }
                (Message::SetKittySize(size), GameState::Initialize(ref mut state)) => {
                    state.set_kitty_size(size)?;
                    let n = s.player_name(id)?;
                    Ok(vec![format!("{} set the kitty to {}", n, size)])
                }
                (Message::SetLandlord(landlord), GameState::Initialize(ref mut state)) => {
                    state.set_landlord(landlord)?;
                    let n = s.player_name(id)?;
                    match landlord {
                        Some(ll) => {
                            let ll_n = s.player_name(ll)?;
                            Ok(vec![format!("{} set the leader to {}", n, ll_n)])
                        }
                        None => Ok(vec![format!(
                            "{} set the leader to the winner of the bid",
                            n
                        )]),
                    }
                }
                (Message::SetGameMode(ref game_mode), GameState::Initialize(ref mut state)) => {
                    state.set_game_mode(game_mode.clone());
                    Ok(vec![])
                }
                (Message::DrawCard, GameState::Draw(ref mut state)) => {
                    state.draw_card(id)?;
                    Ok(vec![])
                }
                (Message::Bid(card, count), GameState::Draw(ref mut state)) => {
                    if state.bid(id, card, count) {
                        let n = s.player_name(id)?;
                        Ok(vec![format!("{} bid {} {:?}", n, count, card)])
                    } else {
                        bail!("bid was invalid")
                    }
                }
                (Message::PickUpKitty, GameState::Draw(ref mut state)) => {
                    *s = GameState::Exchange(state.advance(id)?);
                    Ok(vec![])
                }
                (Message::MoveCardToKitty(card), GameState::Exchange(ref mut state)) => {
                    state.move_card_to_kitty(id, card)?;
                    Ok(vec![])
                }
                (Message::MoveCardToHand(card), GameState::Exchange(ref mut state)) => {
                    state.move_card_to_hand(id, card)?;
                    Ok(vec![])
                }
                (Message::SetFriends(ref friends), GameState::Exchange(ref mut state)) => {
                    state.set_friends(id, friends.iter().cloned())?;
                    Ok(vec![])
                }
                (Message::BeginPlay, GameState::Exchange(ref mut state)) => {
                    *s = GameState::Play(state.advance(id)?);
                    Ok(vec![])
                }
                (Message::PlayCards(ref cards), GameState::Play(ref mut state)) => {
                    state.play_cards(id, cards)?;
                    let n = s.player_name(id)?;
                    let cards_as_str = cards.iter().map(|c| c.as_char()).collect::<String>();
                    Ok(vec![format!("{} played {}", n, cards_as_str)])
                }
                (Message::EndTrick, GameState::Play(ref mut state)) => Ok(state.finish_trick()?),
                (Message::TakeBackCards, GameState::Play(ref mut state)) => {
                    state.take_back_cards(id)?;
                    let n = s.player_name(id)?;
                    Ok(vec![format!("{} took back their last play", n)])
                }
                (Message::StartNewGame, GameState::Play(ref mut state)) => {
                    let (new_s, msgs) = state.finish_game()?;
                    *s = GameState::Initialize(new_s);
                    Ok(msgs)
                }
                _ => bail!("not supported in current phase"),
            }
        } else {
            bail!("lock poisoned")
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    EndGame,
    SetNumDecks(Option<usize>),
    SetKittySize(usize),
    ReorderPlayers(Vec<PlayerID>),
    SetRank(Number),
    SetLandlord(Option<PlayerID>),
    SetGameMode(GameMode),
    StartGame,
    DrawCard,
    Bid(Card, usize),
    PickUpKitty,
    MoveCardToKitty(Card),
    MoveCardToHand(Card),
    SetFriends(Vec<Friend>),
    BeginPlay,
    PlayCards(Vec<Card>),
    EndTrick,
    TakeBackCards,
    StartNewGame,
}
