use std::sync::{Arc, Mutex};

use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};

use crate::game_state::{Friend, GameModeSettings, GameState, InitializePhase, MessageVariant};
use crate::types::{Card, Number, PlayerID};

#[derive(Clone, Debug)]
pub struct InteractiveGame {
    state: Arc<Mutex<GameState>>,
}

impl InteractiveGame {
    pub fn new() -> Self {
        Self::new_from_state(GameState::Initialize(InitializePhase::new()))
    }

    pub fn new_from_state(state: GameState) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn register(
        &self,
        name: String,
    ) -> Result<(PlayerID, Vec<(BroadcastMessage, String)>), Error> {
        if let Ok(mut s) = self.state.lock() {
            let (actor, msgs) = s.register(name)?;
            Ok((
                actor,
                msgs.into_iter()
                    .flat_map(|variant| {
                        let b = BroadcastMessage { actor, variant };
                        b.to_string(|id| s.player_name(id)).ok().map(|s| (b, s))
                    })
                    .collect(),
            ))
        } else {
            bail!("lock poisoned")
        }
    }

    pub fn kick(&self, id: PlayerID) -> Result<Vec<(BroadcastMessage, String)>, Error> {
        if let Ok(mut s) = self.state.lock() {
            let msgs = s.kick(id)?;
            Ok(msgs
                .into_iter()
                .flat_map(|variant| {
                    let b = BroadcastMessage { actor: id, variant };
                    b.to_string(|id| s.player_name(id)).ok().map(|s| (b, s))
                })
                .collect())
        } else {
            bail!("lock poisoned")
        }
    }

    pub fn dump_state(&self) -> Result<GameState, Error> {
        if let Ok(s) = self.state.lock() {
            Ok(s.clone())
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

    pub fn interact(
        &self,
        msg: Message,
        id: PlayerID,
    ) -> Result<Vec<(BroadcastMessage, String)>, Error> {
        if let Ok(mut s) = self.state.lock() {
            let msgs = match (msg, &mut *s) {
                (Message::EndGame, _) => {
                    *s = GameState::Done;
                    vec![]
                }
                (Message::ResetGame, _) => s.reset()?,
                (Message::SetChatLink(ref link), _) => {
                    s.set_chat_link(link.clone())?;
                    vec![]
                }
                (Message::StartGame, GameState::Initialize(ref mut state)) => {
                    *s = GameState::Draw(state.start()?);
                    vec![MessageVariant::StartingGame]
                }
                (Message::ReorderPlayers(ref players), GameState::Initialize(ref mut state)) => {
                    state.reorder_players(&players)?;
                    vec![]
                }
                (Message::MakeObserver(id), GameState::Initialize(ref mut state)) => {
                    state.make_observer(id)?
                }
                (Message::MakePlayer(id), GameState::Initialize(ref mut state)) => {
                    state.make_player(id)?
                }
                (Message::SetNumDecks(num_decks), GameState::Initialize(ref mut state)) => {
                    state.set_num_decks(num_decks)?
                }
                (Message::SetRank(rank), GameState::Initialize(ref mut state)) => {
                    state.set_rank(id, rank)?;
                    vec![MessageVariant::SetRank { rank }]
                }
                (Message::SetKittySize(size), GameState::Initialize(ref mut state)) => {
                    state.set_kitty_size(size)?.into_iter().collect()
                }
                (Message::SetLandlord(landlord), GameState::Initialize(ref mut state)) => {
                    state.set_landlord(landlord)?;
                    vec![MessageVariant::SetLandlord { landlord }]
                }
                (
                    Message::SetHideLandlordsPoints(hide_landlord_points),
                    GameState::Initialize(ref mut state),
                ) => {
                    state.hide_landlord_points(hide_landlord_points);
                    vec![MessageVariant::SetDefendingPointVisibility {
                        visible: !hide_landlord_points,
                    }]
                }
                (Message::SetGameMode(ref game_mode), GameState::Initialize(ref mut state)) => {
                    state.set_game_mode(game_mode.clone())?
                }
                (Message::DrawCard, GameState::Draw(ref mut state)) => {
                    state.draw_card(id)?;
                    vec![]
                }
                (Message::Bid(card, count), GameState::Draw(ref mut state)) => {
                    if state.bid(id, card, count) {
                        vec![MessageVariant::MadeBid { card, count }]
                    } else {
                        bail!("bid was invalid")
                    }
                }
                (Message::PickUpKitty, GameState::Draw(ref mut state)) => {
                    *s = GameState::Exchange(state.advance(id)?);
                    vec![]
                }
                (Message::MoveCardToKitty(card), GameState::Exchange(ref mut state)) => {
                    state.move_card_to_kitty(id, card)?;
                    vec![]
                }
                (Message::MoveCardToHand(card), GameState::Exchange(ref mut state)) => {
                    state.move_card_to_hand(id, card)?;
                    vec![]
                }
                (Message::SetFriends(ref friends), GameState::Exchange(ref mut state)) => {
                    state.set_friends(id, friends.iter().cloned())?;
                    vec![]
                }
                (Message::BeginPlay, GameState::Exchange(ref mut state)) => {
                    *s = GameState::Play(state.advance(id)?);
                    vec![]
                }
                (Message::PlayCards(ref cards), GameState::Play(ref mut state)) => {
                    state.play_cards(id, cards)?;
                    vec![MessageVariant::PlayedCards {
                        cards: cards.to_vec(),
                    }]
                }
                (Message::EndTrick, GameState::Play(ref mut state)) => state.finish_trick()?,
                (Message::TakeBackCards, GameState::Play(ref mut state)) => {
                    state.take_back_cards(id)?;
                    vec![MessageVariant::TookBackPlay]
                }
                (Message::StartNewGame, GameState::Play(ref mut state)) => {
                    let (new_s, msgs) = state.finish_game()?;
                    *s = GameState::Initialize(new_s);
                    msgs
                }
                _ => bail!("not supported in current phase"),
            };

            Ok(msgs
                .into_iter()
                .flat_map(|variant| {
                    let b = BroadcastMessage { actor: id, variant };
                    b.to_string(|id| s.player_name(id)).ok().map(|s| (b, s))
                })
                .collect())
        } else {
            bail!("lock poisoned")
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    EndGame,
    ResetGame,
    MakeObserver(PlayerID),
    MakePlayer(PlayerID),
    SetChatLink(Option<String>),
    SetNumDecks(Option<usize>),
    SetKittySize(Option<usize>),
    SetHideLandlordsPoints(bool),
    ReorderPlayers(Vec<PlayerID>),
    SetRank(Number),
    SetLandlord(Option<PlayerID>),
    SetGameMode(GameModeSettings),
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BroadcastMessage {
    actor: PlayerID,
    variant: MessageVariant,
}

impl BroadcastMessage {
    pub fn to_string<'a>(
        &'a self,
        player_name: impl Fn(PlayerID) -> Result<&'a str, Error>,
    ) -> Result<String, Error> {
        let n = player_name(self.actor);

        use MessageVariant::*;
        Ok(match self.variant {
            ResettingGame => format!("{} reset the game", n?),
            StartingGame => format!("{} started the game", n?),
            TrickWon { winner, points } =>if points > 0 {
                    format!("{} wins the trick and gets {} points", player_name(winner)?, points)
                } else {
                    format!("{} wins the trick, but gets no points :(", player_name(winner)?)
                },
            RankAdvanced { player, new_rank } => format!("{} has advanced to rank {}", player_name(player)?, new_rank.as_str()),
            NewLandlordForNextGame { landlord } => format!("{} will start the next game", player_name(landlord)?),
            PointsInKitty { points, multiplier } => format!("{} points were burried and are attached to the last trick, with a multiplier of {}", points, multiplier),
            JoinedGame { player } => format!("{} has joined the game", player_name(player)?),
            JoinedTeam { player } => format!("{} has joined the team", player_name(player)?),
            LeftGame { ref name } => format!("{} has left the game", name),
            KittySizeSet { size: Some(size) } => format!("{} set the number of cards in the bottom to {}", n?, size),
            KittySizeSet { size: None } => format!("{} set the number of cards in the bottom to default", n?),
            NumDecksSet { num_decks: Some(num_decks) } => format!("{} set the number of decks to {}", n?, num_decks),
            NumDecksSet { num_decks: None } => format!("{} set the number of decks to default", n?),
            NumFriendsSet { num_friends: Some(num_friends) } => format!("{} set the number of friends to {}", n?, num_friends),
            NumFriendsSet { num_friends: None } => format!("{} set the number of friends to default", n?),
            GameModeSet { game_mode: GameModeSettings::Tractor } => format!("{} set the game mode to Tractor", n?),
            GameModeSet { game_mode: GameModeSettings::FindingFriends { num_friends: None }} => format!("{} set the game mode to Finding Friends", n?),
            GameModeSet { game_mode: GameModeSettings::FindingFriends { num_friends: Some(1) }} => format!("{} set the game mode to Finding Friends with 1 friend", n?),
            GameModeSet { game_mode: GameModeSettings::FindingFriends { num_friends: Some(friends) }} => format!("{} set the game mode to Finding Friends with {} friends", n?, friends),
            TookBackPlay => format!("{} took back their last play", n?),
            PlayedCards { ref cards } => format!("{} played {}", n?, cards.iter().map(|c| c.as_char()).collect::<String>()),
            SetDefendingPointVisibility { visible: true } => format!("{} made the defending team's points visible", n?),
            SetDefendingPointVisibility { visible: false } => format!("{} hid the defending team's points", n?),
            SetLandlord { landlord: None } => format!("{} set the leader to the winner of the bid", n?),
            SetLandlord { landlord: Some(landlord) } => format!("{} set the leader to {}", n?, player_name(landlord)?),
            SetRank { rank } => format!("{} set their rank to {}", n?, rank.as_str()),
            MadeBid { card, count } => format!("{} bid {} {:?}", n?, count, card),
        })
    }
}
