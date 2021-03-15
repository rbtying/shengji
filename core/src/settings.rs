use std::collections::HashSet;

use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};
use slog_derive::KV;
use url::Url;

use crate::bidding::{BidPolicy, BidReinforcementPolicy, BidTakebackPolicy, JokerBidPolicy};
use crate::deck::Deck;
use crate::message::MessageVariant;
use crate::player::Player;
use crate::scoring::GameScoringParameters;
use crate::trick::{ThrowEvaluationPolicy, TrickDrawPolicy};
use crate::types::{Card, Number, PlayerID};

#[macro_export]
macro_rules! impl_slog_value {
    ($x: ident) => {
        impl slog::Value for $x {
            fn serialize(
                &self,
                _: &slog::Record,
                key: slog::Key,
                serializer: &mut dyn slog::Serializer,
            ) -> slog::Result {
                serializer.emit_str(key, &format!("{:?}", self))
            }
        }
    };
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Friend {
    pub(crate) card: Card,
    pub(crate) skip: usize,
    pub(crate) initial_skip: usize,
    pub(crate) player_id: Option<PlayerID>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct FriendSelection {
    pub card: Card,
    pub initial_skip: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMode {
    Tractor,
    FindingFriends {
        num_friends: usize,
        friends: Vec<Friend>,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameModeSettings {
    Tractor,
    FindingFriends { num_friends: Option<usize> },
}

impl GameModeSettings {
    pub fn variant(self) -> &'static str {
        match self {
            GameModeSettings::Tractor => "Tractor",
            GameModeSettings::FindingFriends { .. } => "FindingFriends",
        }
    }
}

impl slog::KV for GameModeSettings {
    fn serialize(&self, _: &slog::Record, serializer: &mut dyn slog::Serializer) -> slog::Result {
        match self {
            GameModeSettings::Tractor => serializer.emit_str("game_mode", "Tractor")?,
            GameModeSettings::FindingFriends { num_friends } => {
                serializer.emit_str("game_mode", "FindingFriends")?;
                match num_friends {
                    Some(num_friends) => serializer.emit_usize("num_friends", *num_friends)?,
                    None => serializer.emit_none("num_friends")?,
                }
            }
        }
        Ok(())
    }
}

impl Default for GameModeSettings {
    fn default() -> Self {
        GameModeSettings::Tractor
    }
}

impl_slog_value!(GameModeSettings);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ThrowPenalty {
    None,
    TenPointsPerAttempt,
}

impl Default for ThrowPenalty {
    fn default() -> Self {
        ThrowPenalty::None
    }
}

impl_slog_value!(ThrowPenalty);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum KittyPenalty {
    Times,
    Power,
}

impl Default for KittyPenalty {
    fn default() -> Self {
        KittyPenalty::Times
    }
}

impl_slog_value!(KittyPenalty);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum AdvancementPolicy {
    Unrestricted,
    FullyUnrestricted,
    DefendPoints,
}

impl Default for AdvancementPolicy {
    fn default() -> Self {
        AdvancementPolicy::Unrestricted
    }
}

impl_slog_value!(AdvancementPolicy);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum FriendSelectionPolicy {
    Unrestricted,
    TrumpsIncluded,
    HighestCardNotAllowed,
    PointCardNotAllowed,
}

impl Default for FriendSelectionPolicy {
    fn default() -> Self {
        FriendSelectionPolicy::Unrestricted
    }
}

impl_slog_value!(FriendSelectionPolicy);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum MultipleJoinPolicy {
    Unrestricted,
    NoDoubleJoin,
}

impl Default for MultipleJoinPolicy {
    fn default() -> Self {
        MultipleJoinPolicy::Unrestricted
    }
}

impl_slog_value!(MultipleJoinPolicy);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum FirstLandlordSelectionPolicy {
    ByWinningBid,
    ByFirstBid,
}

impl Default for FirstLandlordSelectionPolicy {
    fn default() -> Self {
        FirstLandlordSelectionPolicy::ByWinningBid
    }
}

impl_slog_value!(FirstLandlordSelectionPolicy);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum KittyBidPolicy {
    FirstCard,
    FirstCardOfLevelOrHighest,
}

impl Default for KittyBidPolicy {
    fn default() -> Self {
        KittyBidPolicy::FirstCard
    }
}

impl_slog_value!(KittyBidPolicy);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PlayTakebackPolicy {
    AllowPlayTakeback,
    NoPlayTakeback,
}

impl Default for PlayTakebackPolicy {
    fn default() -> Self {
        PlayTakebackPolicy::AllowPlayTakeback
    }
}

impl_slog_value!(PlayTakebackPolicy);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum KittyTheftPolicy {
    AllowKittyTheft,
    NoKittyTheft,
}

impl Default for KittyTheftPolicy {
    fn default() -> Self {
        KittyTheftPolicy::NoKittyTheft
    }
}

impl_slog_value!(KittyTheftPolicy);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameShadowingPolicy {
    AllowMultipleSessions,
    SingleSessionOnly,
}

impl Default for GameShadowingPolicy {
    fn default() -> Self {
        GameShadowingPolicy::AllowMultipleSessions
    }
}

impl_slog_value!(GameShadowingPolicy);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameStartPolicy {
    AllowAnyPlayer,
    AllowLandlordOnly,
}

impl Default for GameStartPolicy {
    fn default() -> Self {
        GameStartPolicy::AllowAnyPlayer
    }
}

impl_slog_value!(GameStartPolicy);

#[derive(Debug, Clone, Default, Serialize, Deserialize, KV)]
pub struct PropagatedState {
    #[slog(skip)]
    pub(crate) players: Vec<Player>,
    #[slog(skip)]
    pub(crate) observers: Vec<Player>,
    #[slog(skip)]
    pub(crate) landlord: Option<PlayerID>,
    #[slog(skip)]
    max_player_id: usize,
    #[slog(skip)]
    #[serde(default)]
    pub(crate) num_games_finished: usize,

    pub(crate) game_mode: GameModeSettings,
    #[serde(default)]
    pub(crate) hide_landlord_points: bool,
    pub(crate) kitty_size: Option<usize>,
    #[serde(default)]
    pub(crate) friend_selection_policy: FriendSelectionPolicy,
    #[serde(default)]
    pub(crate) multiple_join_policy: MultipleJoinPolicy,
    pub(crate) num_decks: Option<usize>,
    // TODO: Find a way to log this properly.
    #[slog(skip)]
    #[serde(default)]
    pub(crate) special_decks: Vec<Deck>,
    #[serde(default)]
    pub(crate) landlord_emoji: Option<String>,
    pub(crate) chat_link: Option<String>,
    #[serde(default)]
    pub(crate) advancement_policy: AdvancementPolicy,
    #[serde(default)]
    pub(crate) kitty_penalty: KittyPenalty,
    #[serde(default)]
    pub(crate) throw_penalty: ThrowPenalty,
    #[serde(default)]
    pub(crate) hide_played_cards: bool,
    #[serde(default)]
    pub(crate) kitty_bid_policy: KittyBidPolicy,
    #[serde(default)]
    pub(crate) kitty_theft_policy: KittyTheftPolicy,
    #[serde(default)]
    pub(crate) trick_draw_policy: TrickDrawPolicy,
    #[serde(default)]
    pub(crate) throw_evaluation_policy: ThrowEvaluationPolicy,
    #[serde(default)]
    pub(crate) first_landlord_selection_policy: FirstLandlordSelectionPolicy,
    #[serde(default)]
    pub(crate) bid_policy: BidPolicy,
    #[serde(default)]
    pub(crate) bid_reinforcement_policy: BidReinforcementPolicy,
    #[serde(default)]
    pub(crate) joker_bid_policy: JokerBidPolicy,
    #[serde(default)]
    pub(crate) should_reveal_kitty_at_end_of_game: bool,
    #[serde(default)]
    pub(crate) play_takeback_policy: PlayTakebackPolicy,
    #[serde(default)]
    pub(crate) bid_takeback_policy: BidTakebackPolicy,
    #[serde(default)]
    pub(crate) game_shadowing_policy: GameShadowingPolicy,
    #[serde(default)]
    pub(crate) game_start_policy: GameStartPolicy,
    #[serde(default)]
    pub(crate) game_scoring_parameters: GameScoringParameters,
    #[serde(default)]
    pub(crate) hide_throw_halting_player: bool,
}

impl PropagatedState {
    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn observers(&self) -> &[Player] {
        &self.observers
    }

    pub fn landlord(&self) -> Option<PlayerID> {
        self.landlord
    }

    pub fn trick_draw_policy(&self) -> TrickDrawPolicy {
        self.trick_draw_policy
    }

    pub fn num_decks(&self) -> usize {
        self.num_decks.unwrap_or(self.players.len() / 2)
    }

    pub fn decks(&self) -> Result<Vec<Deck>, Error> {
        let mut decks = self.special_decks.clone();
        let num_decks = self.num_decks();
        if decks.len() > num_decks {
            bail!("More special decks than regular decks?")
        }

        while decks.len() < num_decks {
            decks.push(Deck::default());
        }
        Ok(decks)
    }

    pub fn set_game_mode(
        &mut self,
        game_mode: GameModeSettings,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.game_mode = game_mode;
        Ok(vec![MessageVariant::GameModeSet { game_mode }])
    }

    fn num_players_changed(&mut self) -> Result<Vec<MessageVariant>, Error> {
        let mut msgs = vec![];
        msgs.extend(self.set_num_decks(None)?);

        if let GameModeSettings::FindingFriends {
            ref mut num_friends,
            ..
        } = self.game_mode
        {
            if num_friends.is_some() {
                *num_friends = None;
                msgs.push(MessageVariant::GameModeSet {
                    game_mode: self.game_mode,
                });
            }
        }
        Ok(msgs)
    }

    pub fn add_player(&mut self, name: String) -> Result<(PlayerID, Vec<MessageVariant>), Error> {
        let id = PlayerID(self.max_player_id);
        if self.players.iter().any(|p| p.name == name)
            || self.observers.iter().any(|p| p.name == name)
        {
            bail!("player with name already exists!")
        }

        let mut msgs = vec![MessageVariant::JoinedGame { player: id }];

        self.max_player_id += 1;
        self.players.push(Player::new(id, name));

        msgs.extend(self.num_players_changed()?);
        Ok((id, msgs))
    }

    pub fn reorder_players(&mut self, order: &[PlayerID]) -> Result<(), Error> {
        let uniq = order.iter().cloned().collect::<HashSet<PlayerID>>();
        if uniq.len() != self.players.len() {
            bail!("Incorrect number of players");
        }
        let mut new_players = Vec::with_capacity(self.players.len());
        for id in order {
            match self.players.iter().find(|p| p.id == *id) {
                Some(player) => new_players.push(player.clone()),
                None => bail!("player ID not found"),
            }
        }
        self.players = new_players;
        Ok(())
    }

    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        let id = PlayerID(self.max_player_id);
        if self.players.iter().any(|p| p.name == name)
            || self.observers.iter().any(|p| p.name == name)
        {
            bail!("player with name already exists!")
        }

        self.max_player_id += 1;
        self.observers.push(Player::new(id, name));
        Ok(id)
    }

    pub fn remove_player(&mut self, id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        if let Some(player) = self.players.iter().find(|p| p.id == id).cloned() {
            let mut msgs = vec![MessageVariant::LeftGame { name: player.name }];
            if self.landlord == Some(id) {
                self.landlord = None;
            }
            self.players.retain(|p| p.id != id);
            msgs.extend(self.num_players_changed()?);
            Ok(msgs)
        } else {
            bail!("player not found")
        }
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.observers.retain(|p| p.id != id);
        Ok(())
    }

    pub fn set_chat_link(&mut self, chat_link: Option<String>) -> Result<(), Error> {
        if chat_link.as_ref().map(|link| link.len()).unwrap_or(0) >= 128 {
            bail!("link too long");
        }
        if let Some(ref chat_link) = chat_link {
            if let Ok(u) = Url::parse(&chat_link) {
                if u.scheme() != "https" {
                    bail!("must be https URL")
                }
            } else {
                bail!("Invalid URL")
            }
        }
        self.chat_link = chat_link;
        Ok(())
    }

    pub fn set_special_decks(
        &mut self,
        special_decks: Vec<Deck>,
    ) -> Result<Vec<MessageVariant>, Error> {
        let mut messages = vec![];
        if special_decks.len() > self.num_decks() {
            messages.extend(self.set_num_decks(Some(special_decks.len()))?);
        }
        self.special_decks = special_decks;

        messages.push(MessageVariant::SpecialDecksSet {
            special_decks: self.special_decks.clone(),
        });
        Ok(messages)
    }

    pub fn set_num_decks(
        &mut self,
        num_decks: Option<usize>,
    ) -> Result<Vec<MessageVariant>, Error> {
        if num_decks == Some(0) {
            bail!("At least one deck is necessary to play the game")
        }
        if num_decks.unwrap_or(0) > self.players.len() * 2 {
            bail!("Using more than two decks per player is not supported");
        }
        let mut msgs = vec![];
        if self.num_decks != num_decks {
            msgs.push(MessageVariant::NumDecksSet { num_decks });
            self.num_decks = num_decks;

            if self.special_decks.len() > self.num_decks() {
                self.special_decks.truncate(self.num_decks());
                msgs.push(MessageVariant::SpecialDecksSet {
                    special_decks: self.special_decks.clone(),
                });
            }

            msgs.extend(self.set_kitty_size(None)?);
            if self
                .game_scoring_parameters
                .materialize(&self.decks()?)
                .is_err()
            {
                msgs.extend(self.set_game_scoring_parameters(GameScoringParameters::default())?);
            };
        }
        Ok(msgs)
    }

    pub fn set_kitty_size(
        &mut self,
        kitty_size: Option<usize>,
    ) -> Result<Option<MessageVariant>, Error> {
        if self.kitty_size == kitty_size {
            return Ok(None);
        }
        if let Some(size) = kitty_size {
            if self.players.is_empty() {
                bail!("no players")
            }
            let decks = self.decks()?;
            let deck_len = decks.iter().map(|d| d.len()).sum::<usize>();
            if size >= deck_len {
                bail!("kitty size too large")
            }
            let min = decks.iter().map(|d| d.min).min().unwrap_or(Number::Two);
            let n_decks_with_min = decks.iter().filter(|d| d.includes_number(min)).count();

            // We only allow removing four cards per deck (i.e. one per suit per deck), so check to
            // make sure that things will work out.
            let num_cards_to_remove = (deck_len - size) % self.players.len();
            if num_cards_to_remove > n_decks_with_min * 4 {
                bail!("kitty size requires removing too many cards");
            }

            self.kitty_size = Some(size);
        } else {
            self.kitty_size = None;
        }
        Ok(Some(MessageVariant::KittySizeSet {
            size: self.kitty_size,
        }))
    }

    pub fn set_friend_selection_policy(
        &mut self,
        policy: FriendSelectionPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.friend_selection_policy = policy;
        Ok(vec![MessageVariant::FriendSelectionPolicySet { policy }])
    }

    pub fn set_multiple_join_policy(
        &mut self,
        policy: MultipleJoinPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.multiple_join_policy = policy;
        Ok(vec![MessageVariant::MultipleJoinPolicySet { policy }])
    }

    pub fn set_first_landlord_selection_policy(
        &mut self,
        policy: FirstLandlordSelectionPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.first_landlord_selection_policy = policy;
        Ok(vec![MessageVariant::FirstLandlordSelectionPolicySet {
            policy,
        }])
    }

    pub fn set_bid_policy(&mut self, policy: BidPolicy) -> Result<Vec<MessageVariant>, Error> {
        self.bid_policy = policy;
        Ok(vec![MessageVariant::BidPolicySet { policy }])
    }

    pub fn set_bid_reinforcement_policy(
        &mut self,
        policy: BidReinforcementPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.bid_reinforcement_policy = policy;
        Ok(vec![MessageVariant::BidReinforcementPolicySet { policy }])
    }

    pub fn set_joker_bid_policy(
        &mut self,
        policy: JokerBidPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.joker_bid_policy = policy;
        Ok(vec![MessageVariant::JokerBidPolicySet { policy }])
    }

    pub fn set_should_reveal_kitty_at_end_of_game(
        &mut self,
        should_reveal: bool,
    ) -> Result<Vec<MessageVariant>, Error> {
        self.should_reveal_kitty_at_end_of_game = should_reveal;
        Ok(vec![MessageVariant::ShouldRevealKittyAtEndOfGameSet {
            should_reveal,
        }])
    }

    pub fn set_landlord(&mut self, landlord: Option<PlayerID>) -> Result<(), Error> {
        match landlord {
            Some(landlord) => {
                if self.players.iter().any(|p| p.id == landlord) {
                    self.landlord = Some(landlord)
                } else {
                    bail!("player ID not found")
                }
            }
            None => self.landlord = None,
        }
        Ok(())
    }

    pub fn set_landlord_emoji(&mut self, emoji: Option<String>) -> Result<(), Error> {
        match emoji {
            Some(emoji) => self.landlord_emoji = Some(emoji),
            None => self.landlord_emoji = None,
        }
        Ok(())
    }

    pub fn hide_landlord_points(&mut self, should_hide: bool) -> Result<MessageVariant, Error> {
        self.hide_landlord_points = should_hide;
        Ok(MessageVariant::SetDefendingPointVisibility {
            visible: !should_hide,
        })
    }

    pub fn hide_played_cards(&mut self, should_hide: bool) -> Result<MessageVariant, Error> {
        self.hide_played_cards = should_hide;
        Ok(MessageVariant::SetCardVisibility {
            visible: !should_hide,
        })
    }

    pub fn set_throw_penalty(
        &mut self,
        penalty: ThrowPenalty,
    ) -> Result<Vec<MessageVariant>, Error> {
        if penalty != self.throw_penalty {
            self.throw_penalty = penalty;
            Ok(vec![MessageVariant::ThrowPenaltySet {
                throw_penalty: penalty,
            }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_kitty_penalty(
        &mut self,
        penalty: KittyPenalty,
    ) -> Result<Vec<MessageVariant>, Error> {
        if penalty != self.kitty_penalty {
            self.kitty_penalty = penalty;
            Ok(vec![MessageVariant::KittyPenaltySet {
                kitty_penalty: penalty,
            }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_kitty_bid_policy(
        &mut self,
        policy: KittyBidPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.kitty_bid_policy {
            self.kitty_bid_policy = policy;
            Ok(vec![MessageVariant::KittyBidPolicySet { policy }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_trick_draw_policy(
        &mut self,
        policy: TrickDrawPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.trick_draw_policy {
            self.trick_draw_policy = policy;
            Ok(vec![MessageVariant::TrickDrawPolicySet { policy }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_throw_evaluation_policy(
        &mut self,
        policy: ThrowEvaluationPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.throw_evaluation_policy {
            self.throw_evaluation_policy = policy;
            Ok(vec![MessageVariant::ThrowEvaluationPolicySet { policy }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_play_takeback_policy(
        &mut self,
        policy: PlayTakebackPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.play_takeback_policy {
            self.play_takeback_policy = policy;
            Ok(vec![MessageVariant::PlayTakebackPolicySet { policy }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_bid_takeback_policy(
        &mut self,
        policy: BidTakebackPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.bid_takeback_policy {
            self.bid_takeback_policy = policy;
            Ok(vec![MessageVariant::BidTakebackPolicySet { policy }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_advancement_policy(
        &mut self,
        policy: AdvancementPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.advancement_policy {
            self.advancement_policy = policy;
            Ok(vec![MessageVariant::AdvancementPolicySet { policy }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_game_scoring_parameters(
        &mut self,
        parameters: GameScoringParameters,
    ) -> Result<Vec<MessageVariant>, Error> {
        if parameters != self.game_scoring_parameters {
            let materialized = parameters.materialize(&self.decks()?)?;
            // Explain exercises all the search paths, so make sure to try
            // explaining before accepting the new parameters!
            materialized.explain()?;
            let old_parameters =
                std::mem::replace(&mut self.game_scoring_parameters, parameters.clone());
            let msgs = vec![MessageVariant::GameScoringParametersChanged {
                parameters,
                old_parameters,
            }];
            Ok(msgs)
        } else {
            Ok(vec![])
        }
    }

    pub fn set_kitty_theft_policy(
        &mut self,
        policy: KittyTheftPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.kitty_theft_policy {
            self.kitty_theft_policy = policy;
            Ok(vec![MessageVariant::KittyTheftPolicySet { policy }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_user_multiple_game_session_policy(
        &mut self,
        policy: GameShadowingPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.game_shadowing_policy {
            self.game_shadowing_policy = policy;
            Ok(vec![MessageVariant::GameShadowingPolicySet { policy }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_game_start_policy(
        &mut self,
        policy: GameStartPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.game_start_policy {
            self.game_start_policy = policy;
            Ok(vec![MessageVariant::GameStartPolicySet { policy }])
        } else {
            Ok(vec![])
        }
    }

    pub fn set_hide_throw_halting_player(
        &mut self,
        hide_throw_halting_player: bool,
    ) -> Result<Vec<MessageVariant>, Error> {
        if self.hide_throw_halting_player != hide_throw_halting_player {
            self.hide_throw_halting_player = hide_throw_halting_player;
            Ok(vec![MessageVariant::HideThrowHaltingPlayer {
                set: hide_throw_halting_player,
            }])
        } else {
            Ok(vec![])
        }
    }

    pub fn make_observer(&mut self, player_id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        if let Some(player) = self.players.iter().find(|p| p.id == player_id).cloned() {
            self.players.retain(|p| p.id != player_id);
            if self.landlord == Some(player_id) {
                self.landlord = None;
            }
            self.observers.push(player);
            self.num_players_changed()
        } else {
            bail!("player not found")
        }
    }

    pub fn make_player(&mut self, player_id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        if let Some(player) = self.observers.iter().find(|p| p.id == player_id).cloned() {
            self.observers.retain(|p| p.id != player_id);
            self.players.push(player);
            self.num_players_changed()
        } else {
            bail!("player not found")
        }
    }

    pub fn make_all_observers_into_players(&mut self) -> Result<Vec<MessageVariant>, Error> {
        if self.observers.is_empty() {
            return Ok(vec![]);
        }
        let mut msgs = vec![];
        while let Some(player) = self.observers.pop() {
            msgs.push(MessageVariant::JoinedGame { player: player.id });
            self.players.push(player);
        }
        msgs.extend(self.num_players_changed()?);
        Ok(msgs)
    }

    pub fn set_rank(&mut self, player_id: PlayerID, level: Number) -> Result<(), Error> {
        match self.players.iter_mut().find(|p| p.id == player_id) {
            Some(ref mut player) => {
                player.set_rank(level);
            }
            None => bail!("player not found"),
        }
        Ok(())
    }
}
