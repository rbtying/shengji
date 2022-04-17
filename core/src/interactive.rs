use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};
use slog::{debug, info, o, Logger};

use crate::bidding::{BidPolicy, BidReinforcementPolicy, BidTakebackPolicy, JokerBidPolicy};
use crate::deck::Deck;
use crate::game_state::{initialize_phase::InitializePhase, GameState};
use crate::message::MessageVariant;
use crate::scoring::GameScoringParameters;
use crate::settings::{
    AdvancementPolicy, FirstLandlordSelectionPolicy, FriendSelection, FriendSelectionPolicy,
    GameModeSettings, GameShadowingPolicy, GameStartPolicy, KittyBidPolicy, KittyPenalty,
    KittyTheftPolicy, MultipleJoinPolicy, PlayTakebackPolicy, PropagatedState, ThrowPenalty,
};
use crate::trick::{ThrowEvaluationPolicy, TractorRequirements, TrickDrawPolicy, TrickUnit};
use crate::types::{Card, PlayerID, Rank};

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

    pub fn into_state(self) -> GameState {
        self.state
    }

    pub fn register(
        &mut self,
        name: String,
    ) -> Result<(PlayerID, Vec<(BroadcastMessage, String)>), Error> {
        let (actor, msgs) = self.state.register(name)?;

        Ok((actor, self.hydrate_messages(actor, msgs)?))
    }

    pub fn kick(
        &mut self,
        actor: PlayerID,
        target: PlayerID,
    ) -> Result<Vec<(BroadcastMessage, String)>, Error> {
        let msgs = self.state.kick(target)?;
        self.hydrate_messages(actor, msgs)
    }

    pub fn dump_state(&self) -> Result<GameState, Error> {
        Ok(self.state.clone())
    }

    pub fn allows_multiple_sessions_per_user(&self) -> bool {
        self.state.game_shadowing_policy == GameShadowingPolicy::AllowMultipleSessions
    }

    pub fn dump_state_for_player(&self, id: PlayerID) -> Result<GameState, Error> {
        Ok(self.state.for_player(id))
    }

    pub fn next_player(&self) -> Result<PlayerID, Error> {
        self.state.next_player()
    }

    pub fn player_name(&self, player_id: PlayerID) -> Result<&'_ str, Error> {
        self.state.player_name(player_id)
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn interact(
        &mut self,
        msg: Action,
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
            (Action::ResetGame, _) => {
                info!(logger, "Resetting game");
                self.state.reset()?
            }
            (Action::SetChatLink(ref link), _) => {
                self.state.set_chat_link(link.clone())?;
                vec![]
            }
            (Action::StartGame, GameState::Initialize(ref mut state)) => {
                let s: &'_ PropagatedState = state;
                info!(logger, "Starting game"; s);
                self.state = GameState::Draw(state.start(id)?);
                vec![MessageVariant::StartingGame]
            }
            (Action::ReorderPlayers(ref players), GameState::Initialize(ref mut state)) => {
                info!(logger, "Reordering players");
                state.reorder_players(players)?;
                vec![]
            }
            (Action::MakeObserver(id), GameState::Initialize(ref mut state)) => {
                info!(logger, "Making player an observer"; "id" => id.0);
                state.make_observer(id)?
            }
            (Action::MakePlayer(id), GameState::Initialize(ref mut state)) => {
                info!(logger, "Making observer a player"; "id" => id.0);
                state.make_player(id)?
            }
            (Action::SetNumDecks(num_decks), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting number of decks"; "num_decks" => num_decks);
                state.set_num_decks(num_decks)?
            }
            (Action::SetSpecialDecks(decks), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting special decks"; "decks" => format!("{:?}", decks));
                state.set_special_decks(decks)?
            }
            (Action::SetRank(rank), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting rank"; "rank" => rank.as_str());
                state.set_rank(id, rank)?;
                vec![MessageVariant::SetRank { rank }]
            }
            (Action::SetMetaRank(metarank), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting metarank"; "metarank" => metarank);
                state.set_meta_rank(id, metarank)?;
                vec![MessageVariant::SetMetaRank { metarank }]
            }
            (Action::SetMaxRank(rank), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting max rank"; "max rank" => rank.as_str());
                state.set_max_rank(rank)?;
                vec![MessageVariant::SetMaxRank { rank }]
            }
            (Action::SetKittySize(size), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting kitty size"; "size" => size);
                state.set_kitty_size(size)?.into_iter().collect()
            }
            (Action::SetFriendSelectionPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting friend selection policy"; "policy" => policy);
                state.set_friend_selection_policy(policy)?
            }
            (Action::SetMultipleJoinPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting multiple join policy"; "policy" => policy);
                state.set_multiple_join_policy(policy)?
            }
            (
                Action::SetFirstLandlordSelectionPolicy(policy),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting first landlord selection policy"; "policy" => policy);
                state.set_first_landlord_selection_policy(policy)?
            }
            (Action::SetBidPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting bid selection policy"; "policy" => policy);
                state.set_bid_policy(policy)?
            }
            (Action::SetBidReinforcementPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting bid reinforcement policy"; "policy" => policy);
                state.set_bid_reinforcement_policy(policy)?
            }
            (Action::SetJokerBidPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting joker bid selection policy"; "policy" => policy);
                state.set_joker_bid_policy(policy)?
            }
            (
                Action::SetShouldRevealKittyAtEndOfGame(should_reveal),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting should reveal kitty at end of game"; "should_reveal" => should_reveal);
                state.set_should_reveal_kitty_at_end_of_game(should_reveal)?
            }
            (Action::SetLandlord(landlord), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting landlord"; "landlord" => landlord.map(|l| l.0));
                state.set_landlord(landlord)?;
                vec![MessageVariant::SetLandlord { landlord }]
            }
            (Action::SetLandlordEmoji(ref emoji), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting landlord emoji"; "emoji" => emoji);
                state.set_landlord_emoji(emoji.clone())?;
                vec![MessageVariant::SetLandlordEmoji {
                    emoji: if let Some(a) = emoji {
                        a.to_string()
                    } else {
                        "(当庄)".to_string()
                    },
                }]
            }
            (
                Action::SetHideLandlordsPoints(hide_landlord_points),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting hide landlords points"; "hide_landlord_points" => hide_landlord_points);
                vec![state.hide_landlord_points(hide_landlord_points)?]
            }
            (
                Action::SetHidePlayedCards(hide_played_cards),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting hide played cards"; "hide_played_cards" => hide_played_cards);
                vec![state.hide_played_cards(hide_played_cards)?]
            }
            (
                Action::SetHideThrowHaltingPlayer(hide_throw_halting_player),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting hide throw halting player"; "hide_throw_halting_player" => hide_throw_halting_player);
                state.set_hide_throw_halting_player(hide_throw_halting_player)?
            }
            (Action::SetGameMode(game_mode), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting game mode"; "game_mode" => game_mode.variant());
                state.set_game_mode(game_mode)?
            }
            (Action::SetKittyPenalty(kitty_penalty), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting kitty penalty"; "penalty" => kitty_penalty);
                state.set_kitty_penalty(kitty_penalty)?
            }
            (Action::SetKittyBidPolicy(kitty_bid_policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting kitty bid policy"; "bid_policy" => kitty_bid_policy);
                state.set_kitty_bid_policy(kitty_bid_policy)?
            }
            (Action::SetTrickDrawPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting trick draw policy"; "draw_policy" => policy);
                state.set_trick_draw_policy(policy)?
            }
            (Action::SetAdvancementPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting advancement policy"; "policy" => policy);
                state.set_advancement_policy(policy)?
            }
            (
                Action::SetGameScoringParameters(ref parameters),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting game scoring parameters"; "parameters" => parameters);
                state.set_game_scoring_parameters(parameters.clone())?
            }
            (Action::SetThrowPenalty(throw_penalty), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting throw penalty"; "penalty" => throw_penalty);
                state.set_throw_penalty(throw_penalty)?
            }
            (Action::SetThrowEvaluationPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting throw evaluation policy"; "policy" => policy);
                state.set_throw_evaluation_policy(policy)?
            }
            (Action::SetPlayTakebackPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting play takeback policy"; "policy" => policy);
                state.set_play_takeback_policy(policy)?
            }
            (Action::SetBidTakebackPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting bid takeback policy"; "policy" => policy);
                state.set_bid_takeback_policy(policy)?
            }
            (Action::SetKittyTheftPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting kitty theft policy"; "policy" => policy);
                state.set_kitty_theft_policy(policy)?
            }
            (Action::SetGameShadowingPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting user multiple game session policy"; "policy" => policy);
                state.set_user_multiple_game_session_policy(policy)?
            }
            (Action::SetGameStartPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting game start policy"; "policy" => policy);
                state.set_game_start_policy(policy)?
            }
            (
                Action::SetTractorRequirements(requirements),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting tractor requirements"; "tractor_requirements" => requirements);
                state.set_tractor_requirements(requirements)?
            }
            (Action::DrawCard, GameState::Draw(ref mut state)) => {
                debug!(logger, "Drawing card");
                state.draw_card(id)?;
                vec![]
            }
            (Action::RevealCard, GameState::Draw(ref mut state)) => {
                info!(logger, "Revealing card");
                vec![state.reveal_card()?]
            }
            (Action::Bid(card, count), GameState::Draw(ref mut state)) => {
                info!(logger, "Making bid");
                if state.bid(id, card, count) {
                    vec![MessageVariant::MadeBid { card, count }]
                } else {
                    bail!("bid was invalid")
                }
            }
            (Action::TakeBackBid, GameState::Draw(ref mut state)) => {
                info!(logger, "Taking back bid");
                state.take_back_bid(id)?;
                vec![MessageVariant::TookBackBid]
            }
            (Action::PickUpKitty, GameState::Draw(ref mut state)) => {
                info!(logger, "Entering exchange phase");
                self.state = GameState::Exchange(state.advance(id)?);
                vec![]
            }
            (Action::Bid(card, count), GameState::Exchange(ref mut state)) => {
                info!(logger, "Making exchange bid");
                if state.bid(id, card, count) {
                    vec![MessageVariant::MadeBid { card, count }]
                } else {
                    bail!("bid was invalid")
                }
            }
            (Action::TakeBackBid, GameState::Exchange(ref mut state)) => {
                info!(logger, "Taking back bid");
                state.take_back_bid(id)?;
                vec![MessageVariant::TookBackBid]
            }
            (Action::PickUpKitty, GameState::Exchange(ref mut state)) => {
                info!(logger, "Picking up cards after over-bid");
                state.pick_up_cards(id)?;
                vec![MessageVariant::PickedUpCards]
            }
            (Action::PutDownKitty, GameState::Exchange(ref mut state)) => {
                info!(logger, "Putting down cards after over-bid");
                state.finalize(id)?;
                vec![MessageVariant::PutDownCards]
            }
            (Action::MoveCardToKitty(card), GameState::Exchange(ref mut state)) => {
                info!(logger, "Moving card to kitty");
                state.move_card_to_kitty(id, card)?;
                vec![]
            }
            (Action::MoveCardToHand(card), GameState::Exchange(ref mut state)) => {
                info!(logger, "Moving card to hand");
                state.move_card_to_hand(id, card)?;
                vec![]
            }
            (Action::SetFriends(ref friends), GameState::Exchange(ref mut state)) => {
                info!(logger, "Setting friends");
                state.set_friends(id, friends.iter().cloned())?;
                vec![]
            }
            (Action::BeginPlay, GameState::Exchange(ref mut state)) => {
                info!(logger, "Entering play phase");
                self.state = GameState::Play(state.advance(id)?);
                vec![]
            }
            (Action::PlayCards(ref cards), GameState::Play(ref mut state)) => {
                info!(logger, "Playing cards");
                state.play_cards(id, cards)?
            }
            (
                Action::PlayCardsWithHint(ref cards, ref format_hint),
                GameState::Play(ref mut state),
            ) => {
                info!(logger, "Playing cards with formatting hint");
                state.play_cards_with_hint(id, cards, Some(format_hint))?
            }
            (Action::EndTrick, GameState::Play(ref mut state)) => {
                info!(logger, "Finishing trick");
                state.finish_trick()?
            }
            (Action::TakeBackCards, GameState::Play(ref mut state)) => {
                info!(logger, "Taking back cards");
                state.take_back_cards(id)?;
                vec![MessageVariant::TookBackPlay]
            }
            (Action::EndGameEarly, GameState::Play(ref mut state)) => {
                info!(logger, "Ending game early");
                vec![state.finish_game_early()?]
            }
            (Action::StartNewGame, GameState::Play(ref mut state)) => {
                let s = state.propagated();
                let (new_s, landlord_won, msgs) = state.finish_game()?;
                info!(logger, "Starting new game"; s, "landlord_won_last_game" => landlord_won);
                self.state = GameState::Initialize(new_s);
                msgs
            }
            _ => bail!("not supported in current phase"),
        };

        self.hydrate_messages(id, msgs)
    }

    fn hydrate_messages(
        &self,
        actor: PlayerID,
        msgs: impl IntoIterator<Item = MessageVariant>,
    ) -> Result<Vec<(BroadcastMessage, String)>, Error> {
        let mut out = vec![];
        for msg in msgs {
            let b = BroadcastMessage {
                actor,
                actor_name: self.state.player_name(actor)?.to_owned(),
                variant: msg,
            };
            out.extend(
                b.to_string(|id| self.state.player_name(id))
                    .ok()
                    .map(|s| (b, s)),
            );
        }
        Ok(out)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    ResetGame,
    MakeObserver(PlayerID),
    MakePlayer(PlayerID),
    SetChatLink(Option<String>),
    SetNumDecks(Option<usize>),
    SetSpecialDecks(Vec<Deck>),
    SetKittySize(Option<usize>),
    SetFriendSelectionPolicy(FriendSelectionPolicy),
    SetMultipleJoinPolicy(MultipleJoinPolicy),
    SetFirstLandlordSelectionPolicy(FirstLandlordSelectionPolicy),
    SetBidPolicy(BidPolicy),
    SetBidReinforcementPolicy(BidReinforcementPolicy),
    SetJokerBidPolicy(JokerBidPolicy),
    SetHideLandlordsPoints(bool),
    SetHidePlayedCards(bool),
    ReorderPlayers(Vec<PlayerID>),
    SetRank(Rank),
    SetMetaRank(usize),
    SetMaxRank(Rank),
    SetLandlord(Option<PlayerID>),
    SetLandlordEmoji(Option<String>),
    SetGameMode(GameModeSettings),
    SetAdvancementPolicy(AdvancementPolicy),
    SetGameScoringParameters(GameScoringParameters),
    SetKittyPenalty(KittyPenalty),
    SetKittyBidPolicy(KittyBidPolicy),
    SetTrickDrawPolicy(TrickDrawPolicy),
    SetThrowPenalty(ThrowPenalty),
    SetThrowEvaluationPolicy(ThrowEvaluationPolicy),
    SetPlayTakebackPolicy(PlayTakebackPolicy),
    SetBidTakebackPolicy(BidTakebackPolicy),
    SetKittyTheftPolicy(KittyTheftPolicy),
    SetGameShadowingPolicy(GameShadowingPolicy),
    SetGameStartPolicy(GameStartPolicy),
    SetShouldRevealKittyAtEndOfGame(bool),
    SetHideThrowHaltingPlayer(bool),
    SetTractorRequirements(TractorRequirements),
    StartGame,
    DrawCard,
    RevealCard,
    Bid(Card, usize),
    PickUpKitty,
    PutDownKitty,
    MoveCardToKitty(Card),
    MoveCardToHand(Card),
    SetFriends(Vec<FriendSelection>),
    BeginPlay,
    PlayCards(Vec<Card>),
    PlayCardsWithHint(Vec<Card>, Vec<TrickUnit>),
    EndTrick,
    TakeBackCards,
    TakeBackBid,
    EndGameEarly,
    StartNewGame,
    Beep,
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
        self.variant.to_string(self.actor, player_name)
    }
}
