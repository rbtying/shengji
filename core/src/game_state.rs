use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use anyhow::{anyhow, bail, Error};
use rand::{seq::SliceRandom, RngCore};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::bidding::{Bid, BidPolicy, BidTakebackPolicy};
use crate::hands::Hands;
use crate::message::MessageVariant;
use crate::trick::{ThrowEvaluationPolicy, Trick, TrickDrawPolicy, TrickEnded};
use crate::types::{Card, Number, PlayerID, Trump, FULL_DECK};

macro_rules! bail_unwrap {
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => return Err(anyhow!("option was none")),
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerID,
    pub name: String,
    level: Number,
    metalevel: usize,
}

impl Player {
    pub fn new(id: PlayerID, name: String) -> Player {
        Player {
            id,
            name,
            level: Number::Two,
            metalevel: 1,
        }
    }

    pub fn rank(&self) -> Number {
        self.level
    }

    pub fn set_rank(&mut self, level: Number) {
        self.level = level;
    }

    pub fn advance(&mut self) {
        if let Some(next_level) = self.level.successor() {
            self.level = next_level;
        } else {
            self.metalevel += 1;
            self.level = Number::Two;
        }
    }
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

impl Default for GameModeSettings {
    fn default() -> Self {
        GameModeSettings::Tractor
    }
}

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum AdvancementPolicy {
    Unrestricted,
    DefendPoints,
}

impl Default for AdvancementPolicy {
    fn default() -> Self {
        AdvancementPolicy::Unrestricted
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum FriendSelectionPolicy {
    Unrestricted,
    HighestCardNotAllowed,
}

impl Default for FriendSelectionPolicy {
    fn default() -> Self {
        FriendSelectionPolicy::Unrestricted
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum FirstLandlordSelectionPolicy {
    ByWinningBid,
    ByFirstBid,
}

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

impl Default for FirstLandlordSelectionPolicy {
    fn default() -> Self {
        FirstLandlordSelectionPolicy::ByWinningBid
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BonusLevelPolicy {
    NoBonusLevel,
    BonusLevelForSmallerLandlordTeam,
}

impl Default for BonusLevelPolicy {
    fn default() -> Self {
        BonusLevelPolicy::BonusLevelForSmallerLandlordTeam
    }
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PropagatedState {
    pub game_mode: GameModeSettings,
    #[serde(default)]
    hide_landlord_points: bool,
    kitty_size: Option<usize>,
    #[serde(default)]
    friend_selection_policy: FriendSelectionPolicy,
    num_decks: Option<usize>,
    max_player_id: usize,
    pub players: Vec<Player>,
    pub observers: Vec<Player>,
    landlord: Option<PlayerID>,
    #[serde(default)]
    landlord_emoji: Option<String>,
    chat_link: Option<String>,
    #[serde(default)]
    advancement_policy: AdvancementPolicy,
    #[serde(default)]
    kitty_penalty: KittyPenalty,
    #[serde(default)]
    throw_penalty: ThrowPenalty,
    #[serde(default)]
    hide_played_cards: bool,
    #[serde(default)]
    pub num_games_finished: usize,
    #[serde(default)]
    kitty_bid_policy: KittyBidPolicy,
    #[serde(default)]
    kitty_theft_policy: KittyTheftPolicy,
    #[serde(default)]
    trick_draw_policy: TrickDrawPolicy,
    #[serde(default)]
    throw_evaluation_policy: ThrowEvaluationPolicy,
    #[serde(default)]
    first_landlord_selection_policy: FirstLandlordSelectionPolicy,
    #[serde(default)]
    bid_policy: BidPolicy,
    #[serde(default)]
    bonus_level_policy: BonusLevelPolicy,
    #[serde(default)]
    play_takeback_policy: PlayTakebackPolicy,
    #[serde(default)]
    bid_takeback_policy: BidTakebackPolicy,
}

impl PropagatedState {
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

    pub fn set_num_decks(
        &mut self,
        num_decks: Option<usize>,
    ) -> Result<Vec<MessageVariant>, Error> {
        if num_decks == Some(0) {
            bail!("At least one deck is necessary to play the game")
        }
        let mut msgs = vec![];
        if self.num_decks != num_decks {
            msgs.push(MessageVariant::NumDecksSet { num_decks });
            self.num_decks = num_decks;
            msgs.extend(self.set_kitty_size(None)?);
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
            let deck_len = self.num_decks.unwrap_or(self.players.len() / 2) * FULL_DECK.len();
            if size >= deck_len {
                bail!("kitty size too large")
            }

            if deck_len % self.players.len() != size % self.players.len() {
                bail!("kitty must be a multiple of the remaining cards")
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

    pub fn set_bonus_level_policy(
        &mut self,
        policy: BonusLevelPolicy,
    ) -> Result<Vec<MessageVariant>, Error> {
        if policy != self.bonus_level_policy {
            self.bonus_level_policy = policy;
            Ok(vec![MessageVariant::BonusLevelPolicySet { policy }])
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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Friend {
    card: Card,
    skip: usize,
    initial_skip: usize,
    player_id: Option<PlayerID>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct FriendSelection {
    card: Card,
    initial_skip: usize,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameState {
    Initialize(InitializePhase),
    Draw(DrawPhase),
    Exchange(ExchangePhase),
    Play(PlayPhase),
}

impl GameState {
    pub fn next_player(&self) -> Result<PlayerID, Error> {
        match self {
            GameState::Play(p) => Ok(p.next_player()?),
            GameState::Draw(p) => Ok(p.next_player()?),
            _ => bail!("Not valid in this phase!"),
        }
    }

    pub fn propagated(&self) -> &'_ PropagatedState {
        match self {
            GameState::Initialize(p) => &p.propagated,
            GameState::Draw(p) => &p.propagated,
            GameState::Exchange(p) => &p.propagated,
            GameState::Play(p) => &p.propagated,
        }
    }

    pub fn is_player(&self, id: PlayerID) -> bool {
        self.propagated().players.iter().any(|p| p.id == id)
    }

    pub fn player_name(&self, id: PlayerID) -> Result<&'_ str, Error> {
        for p in &self.propagated().players {
            if p.id == id {
                return Ok(&p.name);
            }
        }
        for p in &self.propagated().observers {
            if p.id == id {
                return Ok(&p.name);
            }
        }
        bail!("Couldn't find player name")
    }

    pub fn player_id(&self, name: &str) -> Result<PlayerID, Error> {
        for p in &self.propagated().players {
            if p.name == name {
                return Ok(p.id);
            }
        }
        for p in &self.propagated().observers {
            if p.name == name {
                return Ok(p.id);
            }
        }
        bail!("Couldn't find player id")
    }

    pub fn cards(&self, id: PlayerID) -> Vec<Card> {
        match self {
            GameState::Initialize { .. } => vec![],
            GameState::Draw(DrawPhase {
                ref hands,
                ref propagated,
                ..
            }) => {
                let level_id = propagated.landlord.unwrap_or(id);
                propagated
                    .players
                    .iter()
                    .filter(|p| p.id == level_id)
                    .flat_map(|p| hands.cards(id, p.level).ok())
                    .next()
                    .unwrap_or_default()
            }
            GameState::Exchange(ExchangePhase {
                ref hands, trump, ..
            })
            | GameState::Play(PlayPhase {
                ref hands, trump, ..
            }) => hands.cards(id, trump.number()).unwrap_or_else(|_| vec![]),
        }
    }

    pub fn register(&mut self, name: String) -> Result<(PlayerID, Vec<MessageVariant>), Error> {
        if let Ok(pid) = self.player_id(&name) {
            return Ok((pid, vec![]));
        }
        match self {
            GameState::Initialize(ref mut p) => p.add_player(name),
            GameState::Draw(ref mut p) => p.add_observer(name).map(|id| (id, vec![])),
            GameState::Exchange(ref mut p) => p.add_observer(name).map(|id| (id, vec![])),
            GameState::Play(ref mut p) => p.add_observer(name).map(|id| (id, vec![])),
        }
    }

    pub fn kick(&mut self, id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        match self {
            GameState::Initialize(ref mut p) => p.remove_player(id),
            GameState::Draw(ref mut p) => p.remove_observer(id).map(|()| vec![]),
            GameState::Exchange(ref mut p) => p.remove_observer(id).map(|()| vec![]),
            GameState::Play(ref mut p) => p.remove_observer(id).map(|()| vec![]),
        }
    }

    pub fn set_chat_link(&mut self, chat_link: Option<String>) -> Result<(), Error> {
        match self {
            GameState::Initialize(ref mut p) => p.propagated.set_chat_link(chat_link),
            GameState::Draw(ref mut p) => p.propagated.set_chat_link(chat_link),
            GameState::Exchange(ref mut p) => p.propagated.set_chat_link(chat_link),
            GameState::Play(ref mut p) => p.propagated.set_chat_link(chat_link),
        }
    }

    pub fn reset(&mut self) -> Result<Vec<MessageVariant>, Error> {
        match self {
            GameState::Initialize(_) => bail!("Game has not started yet!"),
            GameState::Draw(ref mut p) => {
                let (s, m) = p.return_to_initialize()?;
                *self = GameState::Initialize(s);
                Ok(m)
            }
            GameState::Exchange(ref mut p) => {
                let (s, m) = p.return_to_initialize()?;
                *self = GameState::Initialize(s);
                Ok(m)
            }
            GameState::Play(ref mut p) => {
                let (s, m) = p.return_to_initialize()?;
                *self = GameState::Initialize(s);
                Ok(m)
            }
        }
    }

    pub fn for_player(&self, id: PlayerID) -> GameState {
        let mut s = self.clone();
        match s {
            GameState::Initialize { .. } => (),
            GameState::Draw(DrawPhase {
                ref mut hands,
                ref mut kitty,
                ref mut deck,
                revealed_cards,
                ..
            }) => {
                hands.redact_except(id);
                for card in &mut kitty[revealed_cards..] {
                    *card = Card::Unknown;
                }
                for card in deck {
                    *card = Card::Unknown;
                }
            }
            GameState::Exchange(ExchangePhase {
                ref mut hands,
                ref mut kitty,
                ref mut game_mode,
                exchanger,
                landlord,
                finalized,
                ..
            }) => {
                hands.redact_except(id);
                if id != exchanger.unwrap_or(landlord) || finalized {
                    for card in kitty {
                        *card = Card::Unknown;
                    }
                }
                if id != landlord {
                    if let GameMode::FindingFriends {
                        ref mut friends, ..
                    } = game_mode
                    {
                        friends.clear();
                    }
                }
            }
            GameState::Play(PlayPhase {
                ref mut hands,
                ref mut kitty,
                ref mut points,
                ref trick,
                ref landlords_team,
                ref propagated,
                landlord,
                exchanger,
                ..
            }) => {
                if propagated.hide_landlord_points {
                    for (k, v) in points.iter_mut() {
                        if landlords_team.contains(&k) {
                            v.clear();
                        }
                    }
                }
                hands.redact_except(id);
                // Don't redact at the end of the game.
                let game_ongoing = !hands.is_empty() || !trick.played_cards().is_empty();
                if game_ongoing && id != exchanger.unwrap_or(landlord) {
                    for card in kitty {
                        *card = Card::Unknown;
                    }
                }
            }
        }
        s
    }
}

impl Deref for GameState {
    type Target = PropagatedState;

    fn deref(&self) -> &PropagatedState {
        self.propagated()
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct PlayerGameFinishedResult {
    pub won_game: bool,
    pub is_defending: bool,
    pub is_landlord: bool,
    pub ranks_up: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayPhase {
    num_decks: usize,
    game_mode: GameMode,
    propagated: PropagatedState,
    hands: Hands,
    points: HashMap<PlayerID, Vec<Card>>,
    #[serde(default)]
    penalties: HashMap<PlayerID, usize>,
    kitty: Vec<Card>,
    landlord: PlayerID,
    landlords_team: Vec<PlayerID>,
    trump: Trump,
    trick: Trick,
    last_trick: Option<Trick>,
    #[serde(default)]
    exchanger: Option<PlayerID>,
}

impl PlayPhase {
    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

    pub fn next_player(&self) -> Result<PlayerID, Error> {
        Ok(bail_unwrap!(self.trick.next_player()))
    }

    pub fn can_play_cards(&self, id: PlayerID, cards: &[Card]) -> Result<(), Error> {
        Ok(self
            .trick
            .can_play_cards(id, &self.hands, cards, self.propagated.trick_draw_policy)?)
    }

    pub fn play_cards(
        &mut self,
        id: PlayerID,
        cards: &[Card],
    ) -> Result<Vec<MessageVariant>, Error> {
        let mut msgs = self.trick.play_cards(
            id,
            &mut self.hands,
            cards,
            self.propagated.trick_draw_policy,
            self.propagated.throw_evaluation_policy,
        )?;
        if self.propagated.hide_played_cards {
            for msg in &mut msgs {
                match msg {
                    MessageVariant::PlayedCards { ref mut cards, .. } => {
                        for card in cards {
                            *card = Card::Unknown;
                        }
                    }
                    MessageVariant::ThrowFailed {
                        ref mut original_cards,
                        ..
                    } => {
                        for card in original_cards {
                            *card = Card::Unknown;
                        }
                    }
                    _ => (),
                }
            }
        }
        Ok(msgs)
    }

    pub fn take_back_cards(&mut self, id: PlayerID) -> Result<(), Error> {
        if self.propagated.play_takeback_policy == PlayTakebackPolicy::NoPlayTakeback {
            bail!("Taking back played cards is not allowed")
        }
        Ok(self
            .trick
            .take_back(id, &mut self.hands, self.propagated.throw_evaluation_policy)?)
    }

    pub fn finish_trick(&mut self) -> Result<Vec<MessageVariant>, Error> {
        let TrickEnded {
            winner,
            points: mut new_points,
            largest_trick_unit_size,
            failed_throw_size,
        } = self.trick.complete()?;

        let kitty_multipler = match self.propagated.kitty_penalty {
            KittyPenalty::Times => 2 * largest_trick_unit_size,
            KittyPenalty::Power => 2usize.pow(largest_trick_unit_size as u32),
        };

        if failed_throw_size > 0 {
            match self.propagated.throw_penalty {
                ThrowPenalty::None => (),
                ThrowPenalty::TenPointsPerAttempt => {
                    if let Some(id) = self.trick.played_cards().first().map(|pc| pc.id) {
                        *self.penalties.entry(id).or_insert(0) += 10;
                    }
                }
            }
        }

        let mut msgs = vec![];
        if let GameMode::FindingFriends {
            ref mut friends, ..
        } = self.game_mode
        {
            for played in self.trick.played_cards() {
                for card in played.cards.iter() {
                    for friend in friends.iter_mut() {
                        if friend.card == *card {
                            if friend.skip == 0 {
                                if friend.player_id.is_none() {
                                    friend.player_id = Some(played.id);
                                    if !self.landlords_team.contains(&played.id) {
                                        self.landlords_team.push(played.id);
                                        for player in &self.propagated.players {
                                            if player.id == played.id {
                                                msgs.push(MessageVariant::JoinedTeam {
                                                    player: player.id,
                                                });
                                            }
                                        }
                                    }
                                }
                            } else {
                                friend.skip -= 1;
                            }
                        }
                    }
                }
            }
        }
        let points = bail_unwrap!(self.points.get_mut(&winner));
        let kitty_points = self
            .kitty
            .iter()
            .filter(|c| c.points().is_some())
            .copied()
            .collect::<Vec<_>>();

        if self.hands.is_empty() {
            for _ in 0..kitty_multipler {
                new_points.extend(kitty_points.iter().copied());
            }
            if !kitty_points.is_empty() && kitty_multipler > 0 {
                msgs.push(MessageVariant::PointsInKitty {
                    points: kitty_points.iter().flat_map(|c| c.points()).sum::<usize>(),
                    multiplier: kitty_multipler,
                });
            }
        }
        let winner_idx = bail_unwrap!(self.propagated.players.iter().position(|p| p.id == winner));
        if !new_points.is_empty() {
            let trump = self.trump;
            let num_points = new_points.iter().flat_map(|c| c.points()).sum::<usize>();
            points.extend(new_points);
            points.sort_by(|a, b| trump.compare(*a, *b));
            msgs.push(MessageVariant::TrickWon {
                winner: self.propagated.players[winner_idx].id,
                points: num_points,
            });
        } else {
            msgs.push(MessageVariant::TrickWon {
                winner: self.propagated.players[winner_idx].id,
                points: 0,
            });
        }
        let new_trick = Trick::new(
            self.trump,
            (0..self.propagated.players.len()).map(|offset| {
                let idx = (winner_idx + offset) % self.propagated.players.len();
                self.propagated.players[idx].id
            }),
        );
        self.last_trick = Some(std::mem::replace(&mut self.trick, new_trick));

        Ok(msgs)
    }

    pub fn game_finished(&self) -> bool {
        self.hands.is_empty() && self.trick.played_cards().is_empty()
    }

    pub fn compute_player_level_deltas<'a, 'b: 'a>(
        players: impl Iterator<Item = &'b mut Player>,
        non_landlord_level_bump: usize,
        landlord_level_bump: usize,
        landlords_team: &'a [PlayerID],
        landlord_won: bool,
        landlord: PlayerID,
        advancement_policy: AdvancementPolicy,
    ) -> Vec<MessageVariant> {
        let mut msgs = vec![];

        let result = players
            .map(|player| {
                let is_defending = landlords_team.contains(&player.id);
                let bump = if is_defending {
                    landlord_level_bump
                } else {
                    non_landlord_level_bump
                };
                let mut num_advances = 0;
                let mut was_blocked = false;

                for bump_idx in 0..bump {
                    match advancement_policy {
                        // Player *must* defend on Ace and win to advance.
                        _ if player.rank() == Number::Ace && !(is_defending && bump_idx == 0) => {
                            was_blocked = true;
                            break;
                        }
                        AdvancementPolicy::Unrestricted => (),
                        AdvancementPolicy::DefendPoints => match player.rank().points() {
                            None => (),
                            Some(_) if is_defending && bump_idx == 0 => (),
                            Some(_) => {
                                was_blocked = true;
                                break;
                            }
                        },
                    }

                    player.advance();
                    num_advances += 1;
                }
                if num_advances > 0 {
                    msgs.push(MessageVariant::RankAdvanced {
                        player: player.id,
                        new_rank: player.rank(),
                    });
                }
                if was_blocked {
                    msgs.push(MessageVariant::AdvancementBlocked {
                        player: player.id,
                        rank: player.rank(),
                    });
                }

                (
                    player.name.to_string(),
                    PlayerGameFinishedResult {
                        won_game: landlord_won == is_defending,
                        is_defending,
                        is_landlord: landlord == player.id,
                        ranks_up: num_advances,
                    },
                )
            })
            .collect();

        msgs.push(MessageVariant::GameFinished { result });
        msgs
    }

    pub fn finish_game(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        if !self.game_finished() {
            bail!("not done playing yet!")
        }

        let mut msgs = vec![];

        let mut non_landlords_points: isize = self
            .points
            .iter()
            .filter(|(id, _)| !self.landlords_team.contains(id))
            .flat_map(|(_, cards)| cards)
            .flat_map(|c| c.points())
            .sum::<usize>() as isize;

        for (id, penalty) in &self.penalties {
            if *penalty > 0 {
                if self.landlords_team.contains(&id) {
                    non_landlords_points += *penalty as isize;
                } else {
                    non_landlords_points -= *penalty as isize;
                }
            }
        }

        let mut smaller_landlord_team = false;

        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = &self.game_mode
        {
            let actual_team_size: usize;
            let setting_team_size = *num_friends + 1;

            actual_team_size = self.landlords_team.len();
            smaller_landlord_team = actual_team_size < setting_team_size;
        }

        let (non_landlord_level_bump, landlord_level_bump, landlord_won, bonus_level_earned) =
            Self::compute_level_deltas(
                self.num_decks,
                non_landlords_points,
                self.propagated.bonus_level_policy,
                smaller_landlord_team,
            );

        if bonus_level_earned {
            msgs.push(MessageVariant::BonusLevelEarned);
        };

        let mut propagated = self.propagated.clone();

        msgs.extend(Self::compute_player_level_deltas(
            propagated.players.iter_mut(),
            non_landlord_level_bump,
            landlord_level_bump,
            &self.landlords_team[..],
            landlord_won,
            self.landlord,
            propagated.advancement_policy,
        ));

        let landlord_idx = bail_unwrap!(propagated
            .players
            .iter()
            .position(|p| p.id == self.landlord));
        let mut idx = (landlord_idx + 1) % propagated.players.len();
        let (next_landlord, next_landlord_idx) = loop {
            if landlord_won == self.landlords_team.contains(&propagated.players[idx].id) {
                break (propagated.players[idx].id, idx);
            }
            idx = (idx + 1) % propagated.players.len()
        };

        msgs.push(MessageVariant::NewLandlordForNextGame {
            landlord: propagated.players[next_landlord_idx].id,
        });
        propagated.set_landlord(Some(next_landlord))?;
        propagated.num_games_finished += 1;
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase { propagated }, msgs))
    }

    pub fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase { propagated }, msgs))
    }

    pub fn compute_level_deltas(
        num_decks: usize,
        non_landlords_points: isize,
        bonus_level_policy: BonusLevelPolicy,
        smaller_landlord_team_size: bool,
    ) -> (usize, usize, bool, bool) {
        let point_segments = (num_decks * 20) as isize;
        let landlord_won = non_landlords_points < 2 * point_segments;
        let bonus_level;

        if landlord_won
            && bonus_level_policy == BonusLevelPolicy::BonusLevelForSmallerLandlordTeam
            && smaller_landlord_team_size
        {
            bonus_level = 1;
        } else {
            bonus_level = 0;
        };

        if landlord_won && non_landlords_points <= 0 {
            (
                0,
                bonus_level + ((3 - non_landlords_points / point_segments) as usize),
                true,
                bonus_level == 1,
            )
        } else if landlord_won {
            (
                0,
                bonus_level + ((2 - non_landlords_points / point_segments) as usize),
                true,
                bonus_level == 1,
            )
        } else {
            (
                (non_landlords_points / point_segments - 2) as usize,
                0,
                false,
                bonus_level == 1,
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangePhase {
    propagated: PropagatedState,
    num_decks: usize,
    game_mode: GameMode,
    hands: Hands,
    kitty: Vec<Card>,
    kitty_size: usize,
    landlord: PlayerID,
    trump: Trump,
    #[serde(default)]
    exchanger: Option<PlayerID>,
    #[serde(default)]
    finalized: bool,
    #[serde(default)]
    epoch: usize,
    #[serde(default)]
    bids: Vec<Bid>,
    #[serde(default)]
    autobid: Option<Bid>,
}

impl ExchangePhase {
    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

    pub fn move_card_to_kitty(&mut self, id: PlayerID, card: Card) -> Result<(), Error> {
        if self.exchanger.unwrap_or(self.landlord) != id {
            bail!("not the exchanger")
        }
        if self.finalized {
            bail!("cards already finalized")
        }
        self.hands
            .remove(self.exchanger.unwrap_or(self.landlord), Some(card))?;
        self.kitty.push(card);
        Ok(())
    }

    pub fn move_card_to_hand(&mut self, id: PlayerID, card: Card) -> Result<(), Error> {
        if self.exchanger.unwrap_or(self.landlord) != id {
            bail!("not the exchanger")
        }
        if self.finalized {
            bail!("cards already finalized")
        }
        if let Some(index) = self.kitty.iter().position(|c| *c == card) {
            self.kitty.swap_remove(index);
            self.hands
                .add(self.exchanger.unwrap_or(self.landlord), Some(card))?;
            Ok(())
        } else {
            bail!("card not in the kitty")
        }
    }

    pub fn set_friends(
        &mut self,
        id: PlayerID,
        iter: impl IntoIterator<Item = FriendSelection>,
    ) -> Result<(), Error> {
        if self.landlord != id {
            bail!("not the landlord")
        }
        if let GameMode::FindingFriends {
            num_friends,
            ref mut friends,
        } = self.game_mode
        {
            let friend_set = iter.into_iter().collect::<HashSet<_>>();
            if num_friends != friend_set.len() {
                bail!("incorrect number of friends")
            }

            friends.clear();

            for friend in friend_set.iter() {
                if friend.card.is_joker() || friend.card.number() == Some(self.trump.number()) {
                    bail!(
                        "you can't pick a joker or a {} as your friend",
                        self.trump.number().as_str()
                    )
                }
                if self.trump.suit() != None && friend.card.suit() == self.trump.suit() {
                    bail!("you can't pick a trump suit as your friend")
                }
                if friend.initial_skip >= self.num_decks {
                    bail!("need to pick a card that exists!")
                }

                if let FriendSelectionPolicy::HighestCardNotAllowed =
                    self.propagated.friend_selection_policy
                {
                    match (self.trump.number(), friend.card.number()) {
                        (Number::Ace, Some(Number::King)) | (_, Some(Number::Ace)) => {
                            bail!("you can't pick the highest card as your friend")
                        }
                        _ => (),
                    }
                }
                friends.push(Friend {
                    card: friend.card,
                    initial_skip: friend.initial_skip,
                    skip: friend.initial_skip,
                    player_id: None,
                });
            }

            Ok(())
        } else {
            bail!("not playing finding friends")
        }
    }

    pub fn finalize(&mut self, id: PlayerID) -> Result<(), Error> {
        if id != self.exchanger.unwrap_or(self.landlord) {
            bail!("only the exchanger can finalize their cards")
        }
        if self.finalized {
            bail!("Already finalized")
        }
        if self.kitty.len() != self.kitty_size {
            bail!("incorrect number of cards in the bottom")
        }
        self.finalized = true;
        Ok(())
    }

    pub fn pick_up_cards(&mut self, id: PlayerID) -> Result<(), Error> {
        if !self.finalized {
            bail!("Current exchanger is still exchanging cards!")
        }
        if self.autobid.is_some() {
            bail!("Bid was automatically determined; no overbidding allowed")
        }
        if self.bids.last().map(|b| b.epoch) != Some(self.epoch) {
            bail!("No bids have been made since the last player finished exchanging cards")
        }
        let (_, winning_bid) = Bid::first_and_winner(&self.bids, self.autobid)?;
        if id != winning_bid.id {
            bail!("Only the winner of the bid can pick up the cards")
        }
        self.finalized = false;
        self.epoch += 1;
        self.exchanger = Some(winning_bid.id);

        Ok(())
    }

    pub fn bid(&mut self, id: PlayerID, card: Card, count: usize) -> bool {
        if !self.finalized || self.autobid.is_some() {
            return false;
        }
        Bid::bid(
            id,
            card,
            count,
            &mut self.bids,
            self.autobid,
            &self.hands,
            &self.propagated.players,
            self.propagated.landlord,
            self.propagated.bid_policy,
            self.epoch,
        )
    }

    pub fn take_back_bid(&mut self, id: PlayerID) -> Result<(), Error> {
        if !self.finalized {
            bail!("Can't take back bid until exchanger is done swapping cards")
        }
        if self.autobid.is_some() {
            bail!("Can't take back bid if the winning bid was automatic")
        }
        Bid::take_back_bid(
            id,
            self.propagated.bid_takeback_policy,
            &mut self.bids,
            self.epoch,
        )
    }

    pub fn advance(&self, id: PlayerID) -> Result<PlayPhase, Error> {
        if id != self.landlord {
            bail!("only the leader can advance the game")
        }
        if self.kitty.len() != self.kitty_size {
            bail!("incorrect number of cards in the bottom")
        }
        if let GameMode::FindingFriends {
            num_friends,
            ref friends,
        } = self.game_mode
        {
            if friends.len() != num_friends {
                bail!("need to pick friends")
            }
        }

        if self.propagated.kitty_theft_policy == KittyTheftPolicy::AllowKittyTheft
            && self.autobid.is_none()
            && !self.finalized
        {
            bail!("must give other players a chance to over-bid and swap cards")
        }

        let landlord_position = bail_unwrap!(self
            .propagated
            .players
            .iter()
            .position(|p| p.id == self.landlord));
        let landlords_team = match self.game_mode {
            GameMode::Tractor => self
                .propagated
                .players
                .iter()
                .enumerate()
                .flat_map(|(idx, p)| {
                    if idx % 2 == landlord_position % 2 {
                        Some(p.id)
                    } else {
                        None
                    }
                })
                .collect(),
            GameMode::FindingFriends { .. } => vec![self.landlord],
        };
        let landlord_idx = bail_unwrap!(self
            .propagated
            .players
            .iter()
            .position(|p| p.id == self.landlord));

        Ok(PlayPhase {
            num_decks: self.num_decks,
            game_mode: self.game_mode.clone(),
            hands: self.hands.clone(),
            kitty: self.kitty.clone(),
            trick: Trick::new(
                self.trump,
                (0..self.propagated.players.len()).map(|offset| {
                    let idx = (landlord_idx + offset) % self.propagated.players.len();
                    self.propagated.players[idx].id
                }),
            ),
            last_trick: None,
            points: self
                .propagated
                .players
                .iter()
                .map(|p| (p.id, Vec::new()))
                .collect(),
            penalties: self.propagated.players.iter().map(|p| (p.id, 0)).collect(),
            landlord: self.landlord,
            trump: self.trump,
            propagated: self.propagated.clone(),
            exchanger: self.exchanger,
            landlords_team,
        })
    }

    pub fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase { propagated }, msgs))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawPhase {
    num_decks: usize,
    game_mode: GameMode,
    deck: Vec<Card>,
    propagated: PropagatedState,
    hands: Hands,
    bids: Vec<Bid>,
    #[serde(default)]
    autobid: Option<Bid>,
    position: usize,
    kitty: Vec<Card>,
    #[serde(default)]
    revealed_cards: usize,
    level: Option<Number>,
}
impl DrawPhase {
    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

    pub fn next_player(&self) -> Result<PlayerID, Error> {
        if self.deck.is_empty() {
            bail!("Deck has been fully drawn")
        }
        Ok(self.propagated.players[self.position].id)
    }

    pub fn draw_card(&mut self, id: PlayerID) -> Result<(), Error> {
        if id != self.propagated.players[self.position].id {
            bail!("not your turn!");
        }
        if let Some(next_card) = self.deck.pop() {
            self.hands.add(id, Some(next_card))?;
            self.position = (self.position + 1) % self.propagated.players.len();
            Ok(())
        } else {
            bail!("no cards left in deck")
        }
    }

    pub fn reveal_card(&mut self) -> Result<MessageVariant, Error> {
        if !self.deck.is_empty() {
            bail!("can't reveal card until deck is fully drawn")
        }
        if !self.bids.is_empty() {
            bail!("can't reveal card if at least one bid has been made")
        }
        let id = self
            .propagated
            .landlord
            .ok_or_else(|| anyhow!("can't reveal card if landlord hasn't been selected yet"))?;
        if self.revealed_cards >= self.kitty.len() || self.autobid.is_some() {
            bail!("can't reveal any more cards")
        }

        let level = self
            .propagated
            .players
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.rank())
            .ok_or_else(|| anyhow!("can't find landlord level?"))?;

        let card = self.kitty[self.revealed_cards];

        match self.propagated.kitty_bid_policy {
            KittyBidPolicy::FirstCard => {
                self.autobid = Some(Bid {
                    count: 1,
                    id,
                    card,
                    epoch: 0,
                });
            }
            KittyBidPolicy::FirstCardOfLevelOrHighest
                if card.is_joker() || card.number() == Some(level) =>
            {
                self.autobid = Some(Bid {
                    count: 1,
                    id,
                    card,
                    epoch: 0,
                });
            }
            KittyBidPolicy::FirstCardOfLevelOrHighest
                if self.revealed_cards >= self.kitty.len() - 1 =>
            {
                let mut sorted_kitty = self.kitty.clone();
                sorted_kitty.sort_by(|a, b| Trump::NoTrump { number: level }.compare(*a, *b));
                if let Some(highest_card) = sorted_kitty.last() {
                    self.autobid = Some(Bid {
                        count: 1,
                        id,
                        card: *highest_card,
                        epoch: 0,
                    });
                }
            }
            _ => (),
        }
        self.revealed_cards += 1;

        Ok(MessageVariant::RevealedCardFromKitty)
    }

    pub fn bid(&mut self, id: PlayerID, card: Card, count: usize) -> bool {
        if self.revealed_cards > 0 {
            return false;
        }
        Bid::bid(
            id,
            card,
            count,
            &mut self.bids,
            self.autobid,
            &self.hands,
            &self.propagated.players,
            self.propagated.landlord,
            self.propagated.bid_policy,
            0,
        )
    }

    pub fn take_back_bid(&mut self, id: PlayerID) -> Result<(), Error> {
        Bid::take_back_bid(id, self.propagated.bid_takeback_policy, &mut self.bids, 0)
    }

    pub fn advance(&self, id: PlayerID) -> Result<ExchangePhase, Error> {
        if !self.deck.is_empty() {
            bail!("deck has cards remaining")
        } else {
            let (first_bid, winning_bid) = Bid::first_and_winner(&self.bids, self.autobid)?;
            let landlord = self.propagated.landlord.unwrap_or_else(|| {
                match self.propagated.first_landlord_selection_policy {
                    FirstLandlordSelectionPolicy::ByWinningBid => winning_bid.id,
                    FirstLandlordSelectionPolicy::ByFirstBid => first_bid.id,
                }
            });

            if id != landlord {
                bail!("only the leader can advance the game");
            }
            let landlord_level = self
                .propagated
                .players
                .iter()
                .find(|p| p.id == landlord)
                .ok_or_else(|| anyhow!("Couldn't find landlord level?"))?
                .rank();
            let trump = match winning_bid.card {
                Card::Unknown => bail!("can't bid with unknown cards!"),
                Card::SmallJoker | Card::BigJoker => Trump::NoTrump {
                    number: landlord_level,
                },
                Card::Suited { suit, .. } => Trump::Standard {
                    suit,
                    number: landlord_level,
                },
            };
            let mut hands = self.hands.clone();
            hands.set_trump(trump);
            Ok(ExchangePhase {
                num_decks: self.num_decks,
                game_mode: self.game_mode.clone(),
                kitty_size: self.kitty.len(),
                kitty: self.kitty.clone(),
                propagated: self.propagated.clone(),
                landlord,
                hands,
                trump,
                exchanger: None,
                finalized: false,
                epoch: 1,
                bids: self.bids.clone(),
                autobid: self.autobid,
            })
        }
    }

    pub fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase { propagated }, msgs))
    }
}

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

    pub fn start(&self) -> Result<DrawPhase, Error> {
        if self.propagated.players.len() < 4 {
            bail!("not enough players")
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

        let num_decks = self
            .propagated
            .num_decks
            .unwrap_or(self.propagated.players.len() / 2);
        let mut deck = Vec::with_capacity(num_decks * FULL_DECK.len());
        for _ in 0..num_decks {
            deck.extend(FULL_DECK.iter());
        }
        let mut rng = rand::thread_rng();
        deck.shuffle(&mut rng);

        let kitty_size = match self.propagated.kitty_size {
            Some(size)
                if deck.len() % self.propagated.players.len()
                    == size % self.propagated.players.len() =>
            {
                size
            }
            Some(_) => bail!("kitty size doesn't match player count"),
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

        Ok(DrawPhase {
            deck: (&deck[0..deck.len() - kitty_size]).to_vec(),
            kitty: (&deck[deck.len() - kitty_size..]).to_vec(),
            hands: Hands::new(self.propagated.players.iter().map(|p| p.id)),
            propagated: self.propagated.clone(),
            bids: Vec::new(),
            revealed_cards: 0,
            autobid: None,
            position,
            num_decks,
            game_mode,
            level,
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

#[cfg(test)]
mod tests {
    use super::{
        AdvancementPolicy, BonusLevelPolicy, FriendSelection, GameMode, GameModeSettings,
        InitializePhase, KittyTheftPolicy, MessageVariant, PlayPhase, Player,
    };

    use crate::types::{cards, Card, Number, PlayerID};

    #[test]
    fn test_level_deltas() {
        assert_eq!(
            PlayPhase::compute_level_deltas(2, -80, BonusLevelPolicy::NoBonusLevel, false),
            (0, 5, true, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, -40, BonusLevelPolicy::NoBonusLevel, false),
            (0, 4, true, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, -35, BonusLevelPolicy::NoBonusLevel, false),
            (0, 3, true, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 0, BonusLevelPolicy::NoBonusLevel, false),
            (0, 3, true, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 5, BonusLevelPolicy::NoBonusLevel, false),
            (0, 2, true, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 35, BonusLevelPolicy::NoBonusLevel, false),
            (0, 2, true, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 40, BonusLevelPolicy::NoBonusLevel, false),
            (0, 1, true, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 75, BonusLevelPolicy::NoBonusLevel, false),
            (0, 1, true, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 80, BonusLevelPolicy::NoBonusLevel, false),
            (0, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 115, BonusLevelPolicy::NoBonusLevel, false),
            (0, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 120, BonusLevelPolicy::NoBonusLevel, false),
            (1, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 155, BonusLevelPolicy::NoBonusLevel, false),
            (1, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 160, BonusLevelPolicy::NoBonusLevel, false),
            (2, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 195, BonusLevelPolicy::NoBonusLevel, false),
            (2, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 200, BonusLevelPolicy::NoBonusLevel, false),
            (3, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 235, BonusLevelPolicy::NoBonusLevel, false),
            (3, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 240, BonusLevelPolicy::NoBonusLevel, false),
            (4, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(2, 280, BonusLevelPolicy::NoBonusLevel, false),
            (5, 0, false, false)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(
                2,
                0,
                BonusLevelPolicy::BonusLevelForSmallerLandlordTeam,
                true
            ),
            (0, 4, true, true)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(
                3,
                0,
                BonusLevelPolicy::BonusLevelForSmallerLandlordTeam,
                true
            ),
            (0, 4, true, true)
        );
        assert_eq!(
            PlayPhase::compute_level_deltas(
                3,
                50,
                BonusLevelPolicy::BonusLevelForSmallerLandlordTeam,
                true
            ),
            (0, 3, true, true)
        );
    }

    #[test]
    fn test_player_level_deltas() {
        let mut players = vec![
            Player {
                id: PlayerID(0),
                name: "p1".into(),
                level: Number::Four,
                metalevel: 0,
            },
            Player {
                id: PlayerID(1),
                name: "p2".into(),
                level: Number::Four,
                metalevel: 0,
            },
            Player {
                id: PlayerID(2),
                name: "p3".into(),
                level: Number::Four,
                metalevel: 0,
            },
            Player {
                id: PlayerID(3),
                name: "p4".into(),
                level: Number::Four,
                metalevel: 0,
            },
        ];
        let mut players_ = players.clone();

        let _ = PlayPhase::compute_player_level_deltas(
            players.iter_mut(),
            // Pretend both sides are leveling up somehow.
            2,
            2,
            &[PlayerID(0), PlayerID(2)],
            true,
            PlayerID(0),
            AdvancementPolicy::Unrestricted,
        );
        for p in &players {
            assert_eq!(p.rank(), Number::Six);
        }

        let _ = PlayPhase::compute_player_level_deltas(
            players_.iter_mut(),
            // Pretend both sides are leveling up somehow.
            2,
            2,
            &[PlayerID(0), PlayerID(2)],
            true,
            PlayerID(0),
            AdvancementPolicy::DefendPoints,
        );
        for p in &players_ {
            assert_eq!(p.rank(), Number::Five);
        }

        // Advance again!
        let _ = PlayPhase::compute_player_level_deltas(
            players_.iter_mut(),
            // Pretend both sides are leveling up somehow.
            2,
            2,
            &[PlayerID(0), PlayerID(2)],
            true,
            PlayerID(0),
            AdvancementPolicy::DefendPoints,
        );
        for p in &players_ {
            if p.id == PlayerID(0) || p.id == PlayerID(2) {
                assert_eq!(p.rank(), Number::Seven);
            } else {
                assert_eq!(p.rank(), Number::Five);
            }
        }
    }

    #[test]
    fn test_bid_sequence() {
        let mut init = InitializePhase::new();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        let mut draw = init.start().unwrap();
        // Hackily ensure that everyone can bid.
        draw.deck = vec![
            cards::S_2,
            Card::SmallJoker,
            Card::BigJoker,
            cards::H_2,
            cards::S_2,
            Card::SmallJoker,
            Card::BigJoker,
            cards::H_2,
        ];
        draw.position = 0;

        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();
        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();

        assert!(draw.bid(p1, cards::H_2, 1));
        assert!(draw.bid(p1, cards::H_2, 2));
        assert!(draw.bid(p3, Card::SmallJoker, 2));
        assert!(draw.bid(p2, Card::BigJoker, 2));
        assert!(!draw.bid(p1, cards::H_2, 2));
    }

    #[test]
    fn test_kitty_stealing_bid_sequence() {
        let mut init = InitializePhase::new();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        init.set_kitty_theft_policy(KittyTheftPolicy::AllowKittyTheft)
            .unwrap();
        let mut draw = init.start().unwrap();
        // Hackily ensure that everyone can bid.
        draw.deck = vec![
            cards::S_2,
            Card::SmallJoker,
            Card::BigJoker,
            cards::H_2,
            cards::S_2,
            Card::SmallJoker,
            Card::BigJoker,
            cards::H_2,
        ];
        draw.position = 0;

        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();
        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();

        assert!(draw.bid(p1, cards::H_2, 1));
        let mut exchange = draw.advance(p1).unwrap();
        exchange.finalize(p1).unwrap();
        assert!(exchange.bid(p1, cards::H_2, 2));
        assert!(exchange.bid(p3, Card::SmallJoker, 2));
        exchange.pick_up_cards(p3).unwrap();
        exchange.advance(p1).unwrap_err();
        exchange.finalize(p3).unwrap();
        assert!(exchange.bid(p2, Card::BigJoker, 2));
        exchange.pick_up_cards(p2).unwrap();
        exchange.finalize(p2).unwrap();
        assert!(!exchange.bid(p1, cards::H_2, 2));
        exchange.advance(p1).unwrap();
    }

    #[test]
    fn test_tuple_protection_case() {
        use cards::*;

        let mut init = InitializePhase::new();
        init.set_trick_draw_policy(crate::trick::TrickDrawPolicy::LongerTuplesProtected)
            .unwrap();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        let mut draw = init.start().unwrap();

        let p1_hand = vec![S_9, S_9, S_10, S_10, S_K, S_3, S_4, S_5, S_7, S_7, H_2];
        let p2_hand = vec![S_3, S_3, S_5, S_5, S_7, S_8, S_J, S_Q, C_3, C_4, C_5];
        let p3_hand = vec![S_3, S_5, S_10, S_J, S_Q, S_6, S_8, S_8, S_8, C_6, C_7];
        let p4_hand = vec![S_6, S_6, S_6, C_8, C_9, C_10, C_J, C_Q, C_K, C_A, C_A];

        let mut deck = vec![];
        for i in 0..11 {
            deck.push(p1_hand[i]);
            deck.push(p2_hand[i]);
            deck.push(p3_hand[i]);
            deck.push(p4_hand[i]);
        }
        deck.reverse();
        draw.deck = deck;
        draw.position = 0;

        for _ in 0..11 {
            draw.draw_card(p1).unwrap();
            draw.draw_card(p2).unwrap();
            draw.draw_card(p3).unwrap();
            draw.draw_card(p4).unwrap();
        }

        assert!(draw.bid(p1, cards::H_2, 1));

        let exchange = draw.advance(p1).unwrap();
        let mut play = exchange.advance(p1).unwrap();
        play.play_cards(p1, &[S_9, S_9, S_10, S_10, S_K]).unwrap();
        play.play_cards(p2, &[S_3, S_3, S_5, S_5, S_7]).unwrap();
        play.play_cards(p3, &[S_3, S_5, S_10, S_J, S_Q]).unwrap();
        play.play_cards(p4, &[S_6, S_6, S_6, C_8, C_9]).unwrap();
    }

    #[test]
    fn test_full_game_play() {
        use cards::*;

        let mut init = InitializePhase::new();

        init.set_game_mode(GameModeSettings::FindingFriends {
            num_friends: Option::None,
        })
        .unwrap();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        let p5 = init.add_player("p5".into()).unwrap().0;
        let p6 = init.add_player("p6".into()).unwrap().0;

        init.set_landlord(Some(p2)).unwrap();
        init.set_rank(p2, Number::Seven).unwrap();

        let mut draw = init.start().unwrap();

        let p1_hand = vec![
            Card::SmallJoker,
            D_7,
            D_7,
            H_7,
            H_K,
            H_9,
            H_9,
            H_4,
            H_3,
            S_A,
            S_Q,
            S_Q,
            S_9,
            S_9,
            S_8,
            S_5,
            D_K,
            D_8,
            D_6,
            D_5,
            D_4,
            C_K,
            C_K,
            C_J,
            C_9,
            C_8,
        ];
        let p2_hand = vec![
            Card::BigJoker,
            Card::BigJoker,
            C_7,
            C_7,
            S_7,
            H_K,
            H_K,
            H_6,
            H_4,
            H_3,
            S_K,
            S_J,
            S_4,
            S_3,
            S_2,
            D_K,
            D_10,
            D_4,
            D_4,
            D_2,
            D_2,
            C_K,
            C_9,
            C_5,
            C_4,
            C_3,
        ];
        let p3_hand = vec![
            Card::SmallJoker,
            S_7,
            H_A,
            H_10,
            H_10,
            H_8,
            H_8,
            H_5,
            H_5,
            H_2,
            S_10,
            S_8,
            S_5,
            S_3,
            D_A,
            D_J,
            D_8,
            D_6,
            D_5,
            C_A,
            C_J,
            C_10,
            C_6,
            C_5,
            C_5,
            C_2,
        ];
        let p4_hand = vec![
            H_7, S_7, H_Q, H_Q, H_J, H_J, H_8, S_K, S_J, S_10, S_10, S_6, S_2, D_Q, D_8, D_5, D_3,
            D_2, C_A, C_Q, C_J, C_9, C_8, C_6, C_2, C_2,
        ];
        let p5_hand = vec![
            Card::SmallJoker,
            D_7,
            H_A,
            H_9,
            H_6,
            H_3,
            H_2,
            H_2,
            S_K,
            S_6,
            S_6,
            S_5,
            S_4,
            S_2,
            D_Q,
            D_J,
            D_J,
            D_10,
            D_9,
            D_9,
            D_3,
            D_3,
            C_Q,
            C_10,
            C_3,
            C_3,
        ];
        let p6_hand = vec![
            Card::BigJoker,
            H_7,
            H_A,
            H_Q,
            H_10,
            H_6,
            H_5,
            H_4,
            S_A,
            S_A,
            S_Q,
            S_J,
            S_8,
            S_4,
            S_3,
            D_A,
            D_A,
            D_K,
            D_Q,
            D_10,
            D_9,
            C_A,
            C_8,
            C_6,
            C_4,
            C_4,
        ];

        assert_eq!(p1_hand.len(), 26);
        assert_eq!(p2_hand.len(), 26);
        assert_eq!(p3_hand.len(), 26);
        assert_eq!(p4_hand.len(), 26);
        assert_eq!(p5_hand.len(), 26);
        assert_eq!(p6_hand.len(), 26);

        let mut deck = vec![];
        for i in 0..26 {
            deck.push(p1_hand[i]);
            deck.push(p2_hand[i]);
            deck.push(p3_hand[i]);
            deck.push(p4_hand[i]);
            deck.push(p5_hand[i]);
            deck.push(p6_hand[i]);
        }
        deck.reverse();
        draw.deck = deck;
        draw.position = 0;

        for _ in 0..26 {
            draw.draw_card(p1).unwrap();
            draw.draw_card(p2).unwrap();
            draw.draw_card(p3).unwrap();
            draw.draw_card(p4).unwrap();
            draw.draw_card(p5).unwrap();
            draw.draw_card(p6).unwrap();
        }

        draw.kitty = vec![C_7, S_9, D_6, D_J, C_Q, C_10];

        assert!(draw.bid(p1, D_7, 2));

        let mut exchange = draw.advance(p2).unwrap();
        let friends = vec![
            FriendSelection {
                card: C_K,
                initial_skip: 0,
            },
            FriendSelection {
                card: H_K,
                initial_skip: 0,
            },
        ];
        exchange.set_friends(p2, friends).unwrap();
        let mut play = exchange.advance(p2).unwrap();

        assert_eq!(play.landlords_team.len(), 1);

        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p2, &[H_K, H_K]).unwrap();
        play.play_cards(p3, &[H_8, H_8]).unwrap();
        play.play_cards(p4, &[H_J, H_J]).unwrap();
        play.play_cards(p5, &[H_2, H_2]).unwrap();
        play.play_cards(p6, &[H_4, H_5]).unwrap();
        play.play_cards(p1, &[H_9, H_9]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 1);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p2, &[C_3]).unwrap();
        play.play_cards(p3, &[C_6]).unwrap();
        play.play_cards(p4, &[C_6]).unwrap();
        play.play_cards(p5, &[C_10]).unwrap();
        play.play_cards(p6, &[C_6]).unwrap();
        play.play_cards(p1, &[C_K]).unwrap();
        play.finish_trick().unwrap();

        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p1, &[S_A]).unwrap();
        play.play_cards(p2, &[S_2]).unwrap();
        play.play_cards(p3, &[S_3]).unwrap();
        play.play_cards(p4, &[S_2]).unwrap();
        play.play_cards(p5, &[S_2]).unwrap();
        play.play_cards(p6, &[S_3]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p1, &[S_Q, S_Q]).unwrap();
        play.play_cards(p2, &[S_3, S_4]).unwrap();
        play.play_cards(p3, &[S_5, S_8]).unwrap();
        play.play_cards(p4, &[S_10, S_10]).unwrap();
        play.play_cards(p5, &[S_6, S_6]).unwrap();
        play.play_cards(p6, &[S_A, S_A]).unwrap();
        play.finish_trick().unwrap();

        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p6, &[Card::BigJoker]).unwrap();
        play.play_cards(p1, &[D_4]).unwrap();
        play.play_cards(p2, &[S_7]).unwrap();
        play.play_cards(p3, &[D_5]).unwrap();
        play.play_cards(p4, &[D_5]).unwrap();
        play.play_cards(p5, &[D_10]).unwrap();
        play.finish_trick().unwrap();

        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p6, &[D_A, D_A]).unwrap();
        play.play_cards(p1, &[D_7, D_7]).unwrap();
        play.play_cards(p2, &[D_2, D_2]).unwrap();
        play.play_cards(p3, &[D_6, D_8]).unwrap();
        play.play_cards(p4, &[D_2, D_3]).unwrap();
        play.play_cards(p5, &[D_3, D_3]).unwrap();
        play.finish_trick().unwrap();

        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p1, &[S_9, S_9]).unwrap();
        play.play_cards(p2, &[S_J, S_K]).unwrap();
        play.play_cards(p3, &[S_10, H_2]).unwrap();
        play.play_cards(p4, &[S_6, S_J]).unwrap();
        play.play_cards(p5, &[S_4, S_5]).unwrap();
        play.play_cards(p6, &[S_4, S_8]).unwrap();
        play.finish_trick().unwrap();

        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p1, &[S_5]).unwrap();
        play.play_cards(p2, &[D_10]).unwrap();
        play.play_cards(p3, &[C_2]).unwrap();
        play.play_cards(p4, &[S_K]).unwrap();
        play.play_cards(p5, &[S_K]).unwrap();
        play.play_cards(p6, &[S_J]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p2, &[Card::BigJoker, Card::BigJoker])
            .unwrap();
        play.play_cards(p3, &[D_J, D_A]).unwrap();
        play.play_cards(p4, &[D_8, D_Q]).unwrap();
        play.play_cards(p5, &[D_9, D_9]).unwrap();
        play.play_cards(p6, &[D_9, D_10]).unwrap();
        play.play_cards(p1, &[D_5, D_K]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }
        play.play_cards(p2, &[C_7, C_7]).unwrap();
        play.play_cards(p3, &[S_7, Card::SmallJoker]).unwrap();
        play.play_cards(p4, &[S_7, H_7]).unwrap();
        play.play_cards(p5, &[D_J, D_J]).unwrap();
        play.play_cards(p6, &[D_Q, D_K]).unwrap();
        play.play_cards(p1, &[D_6, D_8]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }
        play.play_cards(p2, &[D_4, D_4]).unwrap();
        play.play_cards(p3, &[C_10, C_J]).unwrap();
        play.play_cards(p4, &[C_8, C_9]).unwrap();
        play.play_cards(p5, &[D_Q, D_7]).unwrap();
        play.play_cards(p6, &[C_8, H_7]).unwrap();
        play.play_cards(p1, &[H_7, Card::SmallJoker]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }
        play.play_cards(p2, &[H_3]).unwrap();
        play.play_cards(p3, &[H_A]).unwrap();
        play.play_cards(p4, &[H_8]).unwrap();
        play.play_cards(p5, &[H_3]).unwrap();
        play.play_cards(p6, &[H_6]).unwrap();
        play.play_cards(p1, &[H_3]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p3, &[H_10, H_10]).unwrap();
        play.play_cards(p4, &[H_Q, H_Q]).unwrap();
        play.play_cards(p5, &[H_6, H_9]).unwrap();
        play.play_cards(p6, &[H_10, H_Q]).unwrap();
        play.play_cards(p1, &[H_4, H_K]).unwrap();
        play.play_cards(p2, &[H_4, H_6]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p4, &[C_2]).unwrap();
        play.play_cards(p5, &[C_3]).unwrap();
        play.play_cards(p6, &[C_4]).unwrap();
        play.play_cards(p1, &[C_K]).unwrap();
        play.play_cards(p2, &[C_K]).unwrap();
        play.play_cards(p3, &[C_5]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p1, &[S_8]).unwrap();
        play.play_cards(p2, &[C_4]).unwrap();
        play.play_cards(p3, &[C_A]).unwrap();
        play.play_cards(p4, &[C_A]).unwrap();
        play.play_cards(p5, &[C_3]).unwrap();
        play.play_cards(p6, &[S_Q]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }
        play.play_cards(p6, &[C_4]).unwrap();
        play.play_cards(p1, &[C_8]).unwrap();
        play.play_cards(p2, &[C_9]).unwrap();
        play.play_cards(p3, &[C_5]).unwrap();
        play.play_cards(p4, &[C_2]).unwrap();
        play.play_cards(p5, &[C_Q]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }

        play.play_cards(p5, &[H_A]).unwrap();
        play.play_cards(p6, &[H_A]).unwrap();
        play.play_cards(p1, &[C_9]).unwrap();
        play.play_cards(p2, &[C_5]).unwrap();
        play.play_cards(p3, &[H_5]).unwrap();
        play.play_cards(p4, &[C_J]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }
        play.play_cards(p5, &[Card::SmallJoker]).unwrap();
        play.play_cards(p6, &[C_A]).unwrap();
        play.play_cards(p1, &[C_J]).unwrap();
        play.play_cards(p2, &[D_K]).unwrap();
        play.play_cards(p3, &[H_5]).unwrap();
        play.play_cards(p4, &[C_Q]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team.len(), 2);
        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = play.game_mode
        {
            assert_eq!(num_friends, 2);
        }
        if let Ok((phase, _msgs)) = play.finish_game() {
            assert_eq!(phase.propagated.landlord, Some(p3));
        };
    }

    #[test]
    fn test_landlord_small_team() {
        let mut init = InitializePhase::new();
        init.set_game_mode(GameModeSettings::FindingFriends {
            num_friends: Some(3),
        })
        .unwrap();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        let p5 = init.add_player("p5".into()).unwrap().0;
        let p6 = init.add_player("p6".into()).unwrap().0;
        let p7 = init.add_player("p7".into()).unwrap().0;
        let p8 = init.add_player("p8".into()).unwrap().0;

        init.set_landlord(Some(p1)).unwrap();
        init.set_rank(p1, Number::Seven).unwrap();

        let mut draw = init.start().unwrap();
        let mut deck = vec![];

        // We need at least two cards per person, since the landlord needs to
        // bid, and the biddable card can't be the friend-selection card.
        let p1_hand = vec![cards::S_7, cards::D_3];
        let p2_hand = vec![cards::D_4, cards::D_5];
        let p3_hand = vec![cards::C_6, cards::C_8];
        let p4_hand = vec![cards::C_9, cards::C_10];
        let p5_hand = vec![cards::C_J, cards::C_Q];
        let p6_hand = vec![cards::C_K, cards::C_A];
        let p7_hand = vec![cards::H_2, cards::H_3];
        let p8_hand = vec![cards::H_4, cards::H_5];

        // Set up the deck to have the appropriate cards.
        for i in 0..2 {
            deck.push(p1_hand[i]);
            deck.push(p2_hand[i]);
            deck.push(p3_hand[i]);
            deck.push(p4_hand[i]);
            deck.push(p5_hand[i]);
            deck.push(p6_hand[i]);
            deck.push(p7_hand[i]);
            deck.push(p8_hand[i]);
        }
        deck.reverse();
        draw.deck = deck;
        draw.position = 0;

        // Draw the deck
        for _ in 0..2 {
            draw.draw_card(p1).unwrap();
            draw.draw_card(p2).unwrap();
            draw.draw_card(p3).unwrap();
            draw.draw_card(p4).unwrap();
            draw.draw_card(p5).unwrap();
            draw.draw_card(p6).unwrap();
            draw.draw_card(p7).unwrap();
            draw.draw_card(p8).unwrap();
        }

        // p1 bids and wins, trump is now Spades and 7s.
        assert!(draw.bid(p1, cards::S_7, 1));

        let mut exchange = draw.advance(p1).unwrap();
        let friends = vec![
            FriendSelection {
                card: cards::D_3,
                initial_skip: 0,
            },
            FriendSelection {
                card: cards::D_4,
                initial_skip: 0,
            },
            FriendSelection {
                card: cards::D_5,
                initial_skip: 0,
            },
        ];
        exchange.set_friends(p1, friends).unwrap();
        let mut play = exchange.advance(p1).unwrap();
        match play.game_mode {
            GameMode::FindingFriends { num_friends: 3, .. } => (),
            _ => panic!("Didn't have 3 friends once game was started"),
        }

        assert_eq!(
            play.landlords_team,
            vec![p1],
            "Nobody should have joined the team yet"
        );

        // Play the first hand. P2 will join the team.
        play.play_cards(p1, &p1_hand[..1]).unwrap();
        play.play_cards(p2, &p2_hand[..1]).unwrap();
        play.play_cards(p3, &p3_hand[..1]).unwrap();
        play.play_cards(p4, &p4_hand[..1]).unwrap();
        play.play_cards(p5, &p5_hand[..1]).unwrap();
        play.play_cards(p6, &p6_hand[..1]).unwrap();
        play.play_cards(p7, &p7_hand[..1]).unwrap();
        play.play_cards(p8, &p8_hand[..1]).unwrap();

        // Check that P2 actually joined the team.
        let msgs = play.finish_trick().unwrap();
        assert_eq!(
            msgs.into_iter()
                .filter(|m| match m {
                    MessageVariant::JoinedTeam { player } if *player == p2 => true,
                    _ => false,
                })
                .count(),
            1
        );

        assert_eq!(play.landlords_team, vec![p1, p2]);

        // Play the next trick, where the landlord will join the team, and then
        // p2 will join the team (again).
        play.play_cards(p1, &p1_hand[1..2]).unwrap();
        play.play_cards(p2, &p2_hand[1..2]).unwrap();
        play.play_cards(p3, &p3_hand[1..2]).unwrap();
        play.play_cards(p4, &p4_hand[1..2]).unwrap();
        play.play_cards(p5, &p5_hand[1..2]).unwrap();
        play.play_cards(p6, &p6_hand[1..2]).unwrap();
        play.play_cards(p7, &p7_hand[1..2]).unwrap();
        play.play_cards(p8, &p8_hand[1..2]).unwrap();

        // We don't get any joined-team messages, because both team-joiners have
        // already joined.
        let msgs = play.finish_trick().unwrap();
        assert_eq!(
            msgs.into_iter()
                .filter(|m| match m {
                    MessageVariant::JoinedTeam { .. } => true,
                    _ => false,
                })
                .count(),
            0
        );

        // Assert that the team didn't get any bigger
        assert_eq!(play.landlords_team, vec![p1, p2]);
        // But also that all of the friend cards have been played!
        match play.game_mode {
            GameMode::FindingFriends { ref friends, .. } => assert!(
                friends.iter().all(|f| f.player_id.is_some()),
                "all friends lots taken"
            ),
            _ => unreachable!(),
        }

        // Finish the game; we should see the landlord go up 4 levels (3 for
        // keeping the opposing team at 0, and a bonus level)

        let (new_init_phase, msgs) = play.finish_game().unwrap();
        assert_eq!(
            msgs.into_iter()
                .filter(|m| match m {
                    MessageVariant::BonusLevelEarned => true,
                    MessageVariant::RankAdvanced { player, new_rank } if *player == p1 => {
                        assert_eq!(*new_rank, Number::Jack);
                        false
                    }
                    MessageVariant::RankAdvanced { player, new_rank } if *player == p2 => {
                        assert_eq!(*new_rank, Number::Six);
                        false
                    }
                    _ => false,
                })
                .count(),
            1
        );

        assert_eq!(
            new_init_phase
                .propagated
                .players
                .into_iter()
                .map(|p| p.level)
                .collect::<Vec<Number>>(),
            vec![
                Number::Jack,
                Number::Six,
                Number::Two,
                Number::Two,
                Number::Two,
                Number::Two,
                Number::Two,
                Number::Two
            ],
            "Check that propagated players have the right new levels"
        );
    }
}
