use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};
use slog::{debug, info, o, Logger};

use crate::bidding::{BidPolicy, BidReinforcementPolicy, BidTakebackPolicy, JokerBidPolicy};
use crate::deck::Deck;
use crate::game_state::{GameState, InitializePhase};
use crate::message::MessageVariant;
use crate::scoring::GameScoringParameters;
use crate::settings::{
    AdvancementPolicy, FirstLandlordSelectionPolicy, FriendSelection, FriendSelectionPolicy,
    GameModeSettings, GameShadowingPolicy, GameStartPolicy, KittyBidPolicy, KittyPenalty,
    KittyTheftPolicy, MultipleJoinPolicy, PlayTakebackPolicy, PropagatedState, ThrowPenalty,
};
use crate::trick::{ThrowEvaluationPolicy, TrickDrawPolicy, TrickUnit};
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
        self.hydrate_messages(id, msgs)
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
                state.reorder_players(&players)?;
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
            (Action::SetKittySize(size), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting kitty size"; "size" => size);
                state.set_kitty_size(size)?.into_iter().collect()
            }
            (Action::SetFriendSelectionPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting friend selection policy"; "policy" => format!("{:?}", policy));
                state.set_friend_selection_policy(policy)?
            }
            (Action::SetMultipleJoinPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting multiple join policy"; "policy" => format!("{:?}", policy));
                state.set_multiple_join_policy(policy)?
            }
            (
                Action::SetFirstLandlordSelectionPolicy(policy),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting first landlord selection policy"; "policy" => format!("{:?}", policy));
                state.set_first_landlord_selection_policy(policy)?
            }
            (Action::SetBidPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting bid selection policy"; "policy" => format!("{:?}", policy));
                state.set_bid_policy(policy)?
            }
            (Action::SetBidReinforcementPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting bid reinforcement policy"; "policy" => format!("{:?}", policy));
                state.set_bid_reinforcement_policy(policy)?
            }
            (Action::SetJokerBidPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting joker bid selection policy"; "policy" => format!("{:?}", policy));
                state.set_joker_bid_policy(policy)?
            }
            (
                Action::SetShouldRevealKittyAtEndOfGame(should_reveal),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting should reveal kitty at end of game"; "should_reveal" => format!("{:?}", should_reveal));
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
                info!(logger, "Setting kitty penalty"; "penalty" => format!("{:?}", kitty_penalty));
                state.set_kitty_penalty(kitty_penalty)?
            }
            (Action::SetKittyBidPolicy(kitty_bid_policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting kitty bid policy"; "bid_policy" => format!("{:?}", kitty_bid_policy));
                state.set_kitty_bid_policy(kitty_bid_policy)?
            }
            (Action::SetTrickDrawPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting trick draw policy"; "draw_policy" => format!("{:?}", policy));
                state.set_trick_draw_policy(policy)?
            }
            (Action::SetAdvancementPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting advancement policy"; "policy" => format!("{:?}", policy));
                state.set_advancement_policy(policy)?
            }
            (
                Action::SetGameScoringParameters(ref parameters),
                GameState::Initialize(ref mut state),
            ) => {
                info!(logger, "Setting game scoring parameters"; "parameters" => format!("{:?}", parameters));
                state.set_game_scoring_parameters(parameters.clone())?
            }
            (Action::SetThrowPenalty(throw_penalty), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting throw penalty"; "penalty" => format!("{:?}", throw_penalty));
                state.set_throw_penalty(throw_penalty)?
            }
            (Action::SetThrowEvaluationPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting throw evaluation policy"; "policy" => format!("{:?}", policy));
                state.set_throw_evaluation_policy(policy)?
            }
            (Action::SetPlayTakebackPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting play takeback policy"; "policy" => format!("{:?}", policy));
                state.set_play_takeback_policy(policy)?
            }
            (Action::SetBidTakebackPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting bid takeback policy"; "policy" => format!("{:?}", policy));
                state.set_bid_takeback_policy(policy)?
            }
            (Action::SetKittyTheftPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting kitty theft policy"; "policy" => format!("{:?}", policy));
                state.set_kitty_theft_policy(policy)?
            }
            (Action::SetGameShadowingPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting user multiple game session policy"; "policy" => format!("{:?}", policy));
                state.set_user_multiple_game_session_policy(policy)?
            }
            (Action::SetGameStartPolicy(policy), GameState::Initialize(ref mut state)) => {
                info!(logger, "Setting game start policy"; "policy" => format!("{:?}", policy));
                state.set_game_start_policy(policy)?
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
                debug!(logger, "Taking back bid");
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
                debug!(logger, "Taking back bid");
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
                debug!(logger, "Moving card to kitty");
                state.move_card_to_kitty(id, card)?;
                vec![]
            }
            (Action::MoveCardToHand(card), GameState::Exchange(ref mut state)) => {
                debug!(logger, "Moving card to hand");
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
                debug!(logger, "Playing cards");
                state.play_cards(id, cards)?
            }
            (
                Action::PlayCardsWithHint(ref cards, ref format_hint),
                GameState::Play(ref mut state),
            ) => {
                debug!(logger, "Playing cards with formatting hint");
                state.play_cards_with_hint(id, cards, Some(format_hint))?
            }
            (Action::EndTrick, GameState::Play(ref mut state)) => {
                info!(logger, "Finishing trick");
                state.finish_trick()?
            }
            (Action::TakeBackCards, GameState::Play(ref mut state)) => {
                debug!(logger, "Taking back cards");
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
    SetRank(Number),
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
            AdvancementBlocked { player, rank } => format!("{} must defend on rank {}", player_name(player)?, rank.as_str()),
            NewLandlordForNextGame { landlord } => format!("{} will start the next game", player_name(landlord)?),
            PointsInKitty { points, multiplier } => format!("{} points were buried and are attached to the last trick, with a multiplier of {}", points, multiplier),
            JoinedGame { player } => format!("{} has joined the game", player_name(player)?),
            JoinedGameAgain { player, game_shadowing_policy: GameShadowingPolicy::SingleSessionOnly } => format!("{} has joined the game again, prior connection removed", player_name(player)?),
            JoinedGameAgain { player, game_shadowing_policy: GameShadowingPolicy::AllowMultipleSessions } => format!("{} is being shadowed", player_name(player)?),
            JoinedTeam { player, already_joined: false } => format!("{} has joined the team", player_name(player)?),
            JoinedTeam { player, already_joined: true } => format!("{} tried to join the team, but was already a member", player_name(player)?),
            LeftGame { ref name } => format!("{} has left the game", name),
            AdvancementPolicySet { policy: AdvancementPolicy::FullyUnrestricted } => format!("{} removed all advancement restrictions", n?),
            AdvancementPolicySet { policy: AdvancementPolicy::Unrestricted } => format!("{} required players to defend on A", n?),
            AdvancementPolicySet { policy: AdvancementPolicy::DefendPoints } => format!("{} required players to defend on points and A", n?),
            GameScoringParametersChanged { .. } => format!("{} changed the game's scoring parameters", n?),
            KittySizeSet { size: Some(size) } => format!("{} set the number of cards in the bottom to {}", n?, size),
            KittySizeSet { size: None } => format!("{} set the number of cards in the bottom to default", n?),
            FriendSelectionPolicySet { policy: FriendSelectionPolicy::Unrestricted } => format!("{} allowed any non-trump card to be selected as a friend", n?),
            FriendSelectionPolicySet { policy: FriendSelectionPolicy::TrumpsIncluded } => format!("{} allowed any card to be selected as a friend", n?),
            FriendSelectionPolicySet { policy: FriendSelectionPolicy::HighestCardNotAllowed } => format!("{} disallowed the highest non-trump card, as well as trump cards, from being selected as a friend", n?),
            FriendSelectionPolicySet { policy: FriendSelectionPolicy::PointCardNotAllowed } => format!("{} disallowed point cards, as well as trump cards, from being selected as a friend", n?),
            MultipleJoinPolicySet { policy: MultipleJoinPolicy::Unrestricted } => format!("{} allowed players to join the team multiple times", n?),
            MultipleJoinPolicySet { policy: MultipleJoinPolicy::NoDoubleJoin } => format!("{} prevented players from joining the team multiple times", n?),
            FirstLandlordSelectionPolicySet { policy: FirstLandlordSelectionPolicy::ByWinningBid } => format!("{} set winning bid to decide both landlord and trump", n?),
            FirstLandlordSelectionPolicySet { policy: FirstLandlordSelectionPolicy::ByFirstBid } => format!("{} set first bid to decide landlord, winning bid to decide trump", n?),
            BidPolicySet { policy: BidPolicy::JokerOrGreaterLength } => format!("{} allowed joker bids to outbid non-joker bids with the same number of cards", n?),
            BidPolicySet { policy: BidPolicy::GreaterLength } => format!("{} required all bids to have more cards than the previous bids", n?),
            BidReinforcementPolicySet { policy: BidReinforcementPolicy::ReinforceWhileWinning } => format!("{} allowed reinforcing the winning bid", n?),
            BidReinforcementPolicySet { policy: BidReinforcementPolicy::ReinforceWhileEquivalent } => format!("{} allowed reinforcing bids after they have been overturned", n?),
            BidReinforcementPolicySet { policy: BidReinforcementPolicy::OverturnOrReinforceWhileWinning } => format!("{} allowed overturning your own bids", n?),
            JokerBidPolicySet { policy: JokerBidPolicy::BothNumDecks } => format!("{} required no-trump bids to have every low or high joker", n?),
            JokerBidPolicySet { policy: JokerBidPolicy::LJNumDecksHJNumDecksLessOne } => format!("{} required low no-trump bids to have every low joker (one less required for high joker)", n?),
            JokerBidPolicySet { policy: JokerBidPolicy::BothTwoOrMore } => format!("{} required no-trump bids to have at least two low or high jokers", n?),
            ShouldRevealKittyAtEndOfGameSet { should_reveal: true } => format!("{} enabled the kitty to be revealed at the end of each game", n?),
            ShouldRevealKittyAtEndOfGameSet { should_reveal: false } => format!("{} disabled the kitty from being revealed at the end of each game", n?),
            NumDecksSet { num_decks: Some(num_decks) } => format!("{} set the number of decks to {}", n?, num_decks),
            NumDecksSet { num_decks: None } => format!("{} set the number of decks to default", n?),
            SpecialDecksSet { ref special_decks } if special_decks.is_empty() => format!("{} set the decks to standard 54-card decks", n?),
            SpecialDecksSet { .. } => format!("{} changed the special deck settings", n?),
            NumFriendsSet { num_friends: Some(num_friends) } => format!("{} set the number of friends to {}", n?, num_friends),
            NumFriendsSet { num_friends: None } => format!("{} set the number of friends to default", n?),
            GameModeSet { game_mode: GameModeSettings::Tractor } => format!("{} set the game mode to Tractor", n?),
            GameModeSet { game_mode: GameModeSettings::FindingFriends { num_friends: None }} => format!("{} set the game mode to Finding Friends", n?),
            GameModeSet { game_mode: GameModeSettings::FindingFriends { num_friends: Some(1) }} => format!("{} set the game mode to Finding Friends with 1 friend", n?),
            GameModeSet { game_mode: GameModeSettings::FindingFriends { num_friends: Some(friends) }} => format!("{} set the game mode to Finding Friends with {} friends", n?, friends),
            TookBackBid => format!("{} took back their last bid", n?),
            TookBackPlay => format!("{} took back their last play", n?),
            PlayedCards { ref cards } => format!("{} played {}", n?, cards.iter().map(|c| c.as_char()).collect::<String>()),
            EndOfGameKittyReveal { ref cards } => format!("{} in kitty", cards.iter().map(|c| c.as_char()).collect::<String>()),
            ThrowFailed { ref original_cards, better_player: Some(better_player) } => format!("{} tried to throw {}, but {} can beat it", n?, original_cards.iter().map(|c| c.as_char()).collect::<String>(), player_name(better_player)?),
            ThrowFailed { ref original_cards, better_player: None } => format!("{} tried to throw {}, but someone can beat it", n?, original_cards.iter().map(|c| c.as_char()).collect::<String>()),
            SetDefendingPointVisibility { visible: true } => format!("{} made the defending team's points visible", n?),
            SetDefendingPointVisibility { visible: false } => format!("{} hid the defending team's points", n?),
            SetCardVisibility { visible: true } => format!("{} made the played cards visible in the chat", n?),
            SetCardVisibility { visible: false } => format!("{} hid the played cards from the chat", n?),
            SetLandlord { landlord: None } => format!("{} set the leader to the winner of the bid", n?),
            SetLandlord { landlord: Some(landlord) } => format!("{} set the leader to {}", n?, player_name(landlord)?),
            SetLandlordEmoji { ref emoji } => format!("{} set landlord emoji to {}", n?, *emoji),
            SetRank { rank } => format!("{} set their rank to {}", n?, rank.as_str()),
            MadeBid { card, count } => format!("{} bid {} {:?}", n?, count, card),
            KittyPenaltySet { kitty_penalty: KittyPenalty::Times } => format!("{} set the penalty for points in the bottom to twice the size of the last trick", n?),
            KittyPenaltySet { kitty_penalty: KittyPenalty::Power } => format!("{} set the penalty for points in the bottom to two to the power of the size of the last trick", n?),
            ThrowPenaltySet { throw_penalty: ThrowPenalty::None } => format!("{} removed the throw penalty", n?),
            ThrowPenaltySet { throw_penalty: ThrowPenalty::TenPointsPerAttempt } => format!("{} set the throw penalty to 10 points per throw", n?),
            KittyBidPolicySet { policy: KittyBidPolicy::FirstCard } => format!("{} set the bid-from-bottom policy to be the first card revealed", n?),
            KittyBidPolicySet { policy: KittyBidPolicy::FirstCardOfLevelOrHighest } => format!("{} set the bid-from-bottom policy to be the first card of the appropriate level, or the highest if none are found", n?),
            TrickDrawPolicySet { policy: TrickDrawPolicy::NoProtections } => format!("{} removed all protections (pair can draw triple)", n?),
            TrickDrawPolicySet { policy: TrickDrawPolicy::NoFormatBasedDraw } => format!("{} removed format-based forced-plays (pairs do not draw pairs)", n?),
            TrickDrawPolicySet { policy: TrickDrawPolicy::LongerTuplesProtected } => format!("{} protected longer tuples from being drawn out by shorter ones (pair does not draw triple)", n?),
            TrickDrawPolicySet { policy: TrickDrawPolicy::OnlyDrawTractorOnTractor } => format!("{} protected tractors from being drawn out by non-tractors", n?),
            ThrowEvaluationPolicySet { policy: ThrowEvaluationPolicy::All } => format!("{} set throws to be evaluated based on all of the cards", n?),
            ThrowEvaluationPolicySet { policy: ThrowEvaluationPolicy::Highest } => format!("{} set throws to be evaluated based on the highest card", n?),
            ThrowEvaluationPolicySet { policy: ThrowEvaluationPolicy::TrickUnitLength } => format!("{} set throws to be evaluated based on the longest component", n?),
            PlayTakebackPolicySet { policy: PlayTakebackPolicy::AllowPlayTakeback } => format!("{} allowed taking back plays", n?),
            PlayTakebackPolicySet { policy: PlayTakebackPolicy::NoPlayTakeback } => format!("{} disallowed taking back plays", n?),
            BidTakebackPolicySet { policy: BidTakebackPolicy::AllowBidTakeback } => format!("{} allowed taking back bids", n?),
            BidTakebackPolicySet { policy: BidTakebackPolicy::NoBidTakeback } => format!("{} disallowed taking back bids", n?),
            KittyTheftPolicySet { policy: KittyTheftPolicy::AllowKittyTheft } => format!("{} allowed stealing the bottom cards after the leader", n?),
            KittyTheftPolicySet { policy: KittyTheftPolicy::NoKittyTheft } => format!("{} disabled stealing the bottom cards after the leader", n?),
            GameShadowingPolicySet { policy: GameShadowingPolicy::AllowMultipleSessions } => format!("{} allowed players to be shadowed by joining with the same name", n?),
            GameShadowingPolicySet { policy: GameShadowingPolicy::SingleSessionOnly } => format!("{} prohibited players from being shadowed", n?),
            GameStartPolicySet { policy: GameStartPolicy::AllowAnyPlayer } => format!("{} allowed any player to start a game", n?),
            GameStartPolicySet { policy: GameStartPolicy::AllowLandlordOnly } => format!("{} allowed only landlord to start a game", n?),
            RevealedCardFromKitty => format!("{} revealed a card from the bottom of the deck", n?),
            PickedUpCards => format!("{} picked up the bottom cards", n?),
            PutDownCards => format!("{} put down the bottom cards", n?),
            GameFinished { result: _ } => "The game has finished".to_string(),
            GameEndedEarly => format!("{} ended the game early", n?),
            BonusLevelEarned => "Landlord team earned a bonus level for defending with a smaller team".to_string(),
            EndOfGameSummary { landlord_won : true, non_landlords_points } => format!("Landlord team won, opposing team only collected {} points", non_landlords_points),
            EndOfGameSummary { landlord_won: false, non_landlords_points } => format!("Landlord team lost, opposing team collected {} points", non_landlords_points),
            HideThrowHaltingPlayer { set: true } => format!("{} hid the player who prevents throws", n?),
            HideThrowHaltingPlayer { set: false } => format!("{} un-hid the player who prevents throws", n?),
        })
    }
}
