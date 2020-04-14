use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};
use slog::{debug, info, o, Logger};

use crate::game_state::{
    Friend, GameModeSettings, GameState, InitializePhase, KittyPenalty, ThrowPenalty,
};
use crate::message::MessageVariant;
use crate::types::{Card, Number, PlayerID};

pub struct InteractiveGame {
    state: GameState,
}

impl InteractiveGame {
    pub fn new() -> Self {
        Self::new_from_state(GameState::Initialize(InitializePhase::new()))
    }

    pub fn new_from_state(state: GameState) -> Self {
        Self { state }
    }

    pub fn register(
        &mut self,
        name: String,
    ) -> Result<(PlayerID, Vec<(BroadcastMessage, String)>), Error> {
        let (actor, msgs) = self.state.register(name)?;
        Ok((actor, self.hydrate_messages(actor, msgs)?))
    }

    pub fn kick(&mut self, id: PlayerID) -> Result<Vec<(BroadcastMessage, String)>, Error> {
        let msgs = self.state.kick(id)?;
        Ok(self.hydrate_messages(id, msgs)?)
    }

    pub fn dump_state(&self) -> Result<GameState, Error> {
        Ok(self.state.clone())
    }

    pub fn dump_state_for_player(&self, id: PlayerID) -> Result<(GameState, Vec<Card>), Error> {
        Ok((self.state.for_player(id), self.state.cards(id)))
    }

    pub fn interact(
        &mut self,
        msg: Message,
        id: PlayerID,
        logger: &Logger,
    ) -> Result<Vec<(BroadcastMessage, String)>, Error> {
        let logger = logger.new(o!(
            "num_players" => self.state.players.len(),
            "num_observers" => self.state.observers.len(),
            "mode" => self.state.game_mode.variant(),
            "num_games_finished" => self.state.num_games_finished,
        ));

        let msgs = match (msg, &mut self.state) {
            (Message::ResetGame, _) => {
                info!(logger, "Resetting game");
                self.state.reset()?
            }
            (Message::SetChatLink(ref link), _) => {
                self.state.set_chat_link(link.clone())?;
                vec![]
            }
            (Message::StartGame, GameState::Initialize(ref mut state)) => {
                info!(logger, "Starting game");
                self.state = GameState::Draw(state.start()?);
                vec![MessageVariant::StartingGame]
            }
            (Message::ReorderPlayers(ref players), GameState::Initialize(ref mut state)) => {
                info!(logger, "Reordering players");
                state.reorder_players(&players)?;
                vec![]
            }
            (Message::MakeObserver(id), GameState::Initialize(ref mut state)) => {
                info!(logger, "Making player an observer"; "id" => id.0);
                state.make_observer(id)?
            }
            (Message::MakePlayer(id), GameState::Initialize(ref mut state)) => {
                info!(logger, "Making observer a player"; "id" => id.0);
                state.make_player(id)?
            }
            (Message::SetNumDecks(num_decks), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting number of decks"; "num_decks" => num_decks);
                state.set_num_decks(num_decks)?
            }
            (Message::SetRank(rank), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting rank"; "rank" => rank.as_str());
                state.set_rank(id, rank)?;
                vec![MessageVariant::SetRank { rank }]
            }
            (Message::SetKittySize(size), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting kitty size"; "size" => size);
                state.set_kitty_size(size)?.into_iter().collect()
            }
            (Message::SetLandlord(landlord), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting landlord"; "landlord" => landlord.map(|l| l.0));
                state.set_landlord(landlord)?;
                vec![MessageVariant::SetLandlord { landlord }]
            }
            (
                Message::SetHideLandlordsPoints(hide_landlord_points),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting hide landlords points"; "hide_landlord_points" => hide_landlord_points);
                vec![state.hide_landlord_points(hide_landlord_points)?]
            }
            (
                Message::SetHidePlayedCards(hide_played_cards),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting hide played cards"; "hide_played_cards" => hide_played_cards);
                vec![state.hide_played_cards(hide_played_cards)?]
            }
            (Message::SetGameMode(ref game_mode), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting game mode"; "game_mode" => game_mode.variant());
                state.set_game_mode(game_mode.clone())?
            }
            (Message::SetKittyPenalty(kitty_penalty), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting kitty penalty"; "penalty" => format!("{:?}", kitty_penalty));
                state.set_kitty_penalty(kitty_penalty)?
            }
            (Message::SetThrowPenalty(throw_penalty), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting throw penalty"; "penalty" => format!("{:?}", throw_penalty));
                state.set_throw_penalty(throw_penalty)?
            }
            (Message::DrawCard, GameState::Draw(ref mut state)) => {
                debug!(logger, "Drawing card");
                state.draw_card(id)?;
                vec![]
            }
            (Message::Bid(card, count), GameState::Draw(ref mut state)) => {
                info!(logger, "Making bid");
                if state.bid(id, card, count) {
                    vec![MessageVariant::MadeBid { card, count }]
                } else {
                    bail!("bid was invalid")
                }
            }
            (Message::PickUpKitty, GameState::Draw(ref mut state)) => {
                info!(logger, "Entering exchange phase");
                self.state = GameState::Exchange(state.advance(id)?);
                vec![]
            }
            (Message::MoveCardToKitty(card), GameState::Exchange(ref mut state)) => {
                debug!(logger, "Moving card to kitty");
                state.move_card_to_kitty(id, card)?;
                vec![]
            }
            (Message::MoveCardToHand(card), GameState::Exchange(ref mut state)) => {
                debug!(logger, "Moving card to hand");
                state.move_card_to_hand(id, card)?;
                vec![]
            }
            (Message::SetFriends(ref friends), GameState::Exchange(ref mut state)) => {
                info!(logger, "Setting friends");
                state.set_friends(id, friends.iter().cloned())?;
                vec![]
            }
            (Message::BeginPlay, GameState::Exchange(ref mut state)) => {
                info!(logger, "Entering play phase");
                self.state = GameState::Play(state.advance(id)?);
                vec![]
            }
            (Message::PlayCards(ref cards), GameState::Play(ref mut state)) => {
                debug!(logger, "Playing cards");
                state.play_cards(id, cards)?
            }
            (Message::EndTrick, GameState::Play(ref mut state)) => {
                info!(logger, "Finishing trick");
                state.finish_trick()?
            }
            (Message::TakeBackCards, GameState::Play(ref mut state)) => {
                debug!(logger, "Taking back cards");
                state.take_back_cards(id)?;
                vec![MessageVariant::TookBackPlay]
            }
            (Message::StartNewGame, GameState::Play(ref mut state)) => {
                info!(logger, "Starting new game");
                let (new_s, msgs) = state.finish_game()?;
                self.state = GameState::Initialize(new_s);
                msgs
            }
            _ => bail!("not supported in current phase"),
        };

        Ok(self.hydrate_messages(id, msgs)?)
    }

    fn hydrate_messages(
        &self,
        actor: PlayerID,
        msgs: impl IntoIterator<Item = MessageVariant>,
    ) -> Result<Vec<(BroadcastMessage, String)>, Error> {
        Ok(msgs
            .into_iter()
            .flat_map(|variant| {
                let b = BroadcastMessage {
                    actor,
                    actor_name: self.state.player_name(actor).ok()?.to_owned(),
                    variant,
                };
                b.to_string(|id| self.state.player_name(id))
                    .ok()
                    .map(|s| (b, s))
            })
            .collect())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    ResetGame,
    MakeObserver(PlayerID),
    MakePlayer(PlayerID),
    SetChatLink(Option<String>),
    SetNumDecks(Option<usize>),
    SetKittySize(Option<usize>),
    SetHideLandlordsPoints(bool),
    SetHidePlayedCards(bool),
    ReorderPlayers(Vec<PlayerID>),
    SetRank(Number),
    SetLandlord(Option<PlayerID>),
    SetGameMode(GameModeSettings),
    SetKittyPenalty(KittyPenalty),
    SetThrowPenalty(ThrowPenalty),
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
    actor_name: String,
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
            PointsInKitty { points, multiplier } => format!("{} points were buried and are attached to the last trick, with a multiplier of {}", points, multiplier),
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
            ThrowFailed { ref original_cards, better_player } => format!("{} tried to throw {}, but {} can beat it", n?, original_cards.iter().map(|c| c.as_char()).collect::<String>(), player_name(better_player)?),
            SetDefendingPointVisibility { visible: true } => format!("{} made the defending team's points visible", n?),
            SetDefendingPointVisibility { visible: false } => format!("{} hid the defending team's points", n?),
            SetCardVisibility { visible: true } => format!("{} made the played cards visible in the chat", n?),
            SetCardVisibility { visible: false } => format!("{} hid the played cards from the chat", n?),
            SetLandlord { landlord: None } => format!("{} set the leader to the winner of the bid", n?),
            SetLandlord { landlord: Some(landlord) } => format!("{} set the leader to {}", n?, player_name(landlord)?),
            SetRank { rank } => format!("{} set their rank to {}", n?, rank.as_str()),
            MadeBid { card, count } => format!("{} bid {} {:?}", n?, count, card),
            KittyPenaltySet { kitty_penalty: KittyPenalty::Times } => format!("{} set the penalty for points in the bottom to twice the size of the last trick", n?),
            KittyPenaltySet { kitty_penalty: KittyPenalty::Power } => format!("{} set the penalty for points in the bottom to two to the power of the size of the last trick", n?),
            ThrowPenaltySet { throw_penalty: ThrowPenalty::None } => format!("{} removed the throw penalty", n?),
            ThrowPenaltySet { throw_penalty: ThrowPenalty::TenPointsPerAttempt } => format!("{} set the throw penalty to 10 points per throw", n?),
        })
    }
}
