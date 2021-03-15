use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use anyhow::{anyhow, bail, Error};
use rand::{seq::SliceRandom, RngCore};
use serde::{Deserialize, Serialize};

use crate::bidding::Bid;
use crate::deck::Deck;
use crate::hands::Hands;
use crate::message::MessageVariant;
use crate::player::Player;
use crate::scoring::{compute_level_deltas, next_threshold_reachable, GameScoreResult};
use crate::settings::{
    AdvancementPolicy, FirstLandlordSelectionPolicy, Friend, FriendSelection,
    FriendSelectionPolicy, GameMode, GameModeSettings, GameStartPolicy, KittyBidPolicy,
    KittyPenalty, KittyTheftPolicy, MultipleJoinPolicy, PlayTakebackPolicy, PropagatedState,
    ThrowPenalty,
};
use crate::trick::{PlayCards, Trick, TrickEnded, TrickUnit};
use crate::types::{Card, Number, PlayerID, Trump, ALL_SUITS};

macro_rules! bail_unwrap {
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => return Err(anyhow!("option was none")),
        }
    };
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
            GameState::Exchange(p) => Ok(p.next_player()?),
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

    pub fn register(&mut self, name: String) -> Result<(PlayerID, Vec<MessageVariant>), Error> {
        if let Ok(pid) = self.player_id(&name) {
            return Ok((
                pid,
                vec![MessageVariant::JoinedGameAgain {
                    player: pid,
                    game_shadowing_policy: self.game_shadowing_policy,
                }],
            ));
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
                game_ended_early,
                ..
            }) => {
                if propagated.hide_landlord_points {
                    for (k, v) in points.iter_mut() {
                        if landlords_team.contains(&k) {
                            v.clear();
                        }
                    }
                }
                // Don't redact at the end of the game.
                let game_ongoing =
                    !game_ended_early && (!hands.is_empty() || !trick.played_cards().is_empty());
                if game_ongoing {
                    hands.redact_except(id);
                }
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
    pub confetti: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayPhase {
    num_decks: usize,
    game_mode: GameMode,
    propagated: PropagatedState,
    hands: Hands,
    points: HashMap<PlayerID, Vec<Card>>,
    penalties: HashMap<PlayerID, usize>,
    kitty: Vec<Card>,
    landlord: PlayerID,
    landlords_team: Vec<PlayerID>,
    trump: Trump,
    trick: Trick,
    last_trick: Option<Trick>,
    exchanger: Option<PlayerID>,
    game_ended_early: bool,
    #[serde(default)]
    removed_cards: Vec<Card>,
    #[serde(default)]
    decks: Vec<Deck>,
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

    pub fn trick(&self) -> &Trick {
        &self.trick
    }

    pub fn hands(&self) -> &Hands {
        &self.hands
    }

    pub fn propagated(&self) -> &PropagatedState {
        &self.propagated
    }

    pub fn can_play_cards(&self, id: PlayerID, cards: &[Card]) -> Result<(), Error> {
        if self.game_ended_early {
            bail!("Game has already ended; cards can't be played");
        }
        Ok(self
            .trick
            .can_play_cards(id, &self.hands, cards, self.propagated.trick_draw_policy)?)
    }

    pub fn play_cards(
        &mut self,
        id: PlayerID,
        cards: &[Card],
    ) -> Result<Vec<MessageVariant>, Error> {
        self.play_cards_with_hint(id, cards, None)
    }

    pub fn play_cards_with_hint(
        &mut self,
        id: PlayerID,
        cards: &[Card],
        format_hint: Option<&'_ [TrickUnit]>,
    ) -> Result<Vec<MessageVariant>, Error> {
        if self.game_ended_early {
            bail!("Game has already ended; cards can't be played");
        }

        let mut msgs = self.trick.play_cards(PlayCards {
            id,
            hands: &mut self.hands,
            cards,
            trick_draw_policy: self.propagated.trick_draw_policy,
            throw_eval_policy: self.propagated.throw_evaluation_policy,
            format_hint,
            hide_throw_halting_player: self.propagated.hide_throw_halting_player,
        })?;
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
        if self.game_ended_early {
            bail!("Game has already ended; cards can't be taken back");
        }
        if self.propagated.play_takeback_policy == PlayTakebackPolicy::NoPlayTakeback {
            bail!("Taking back played cards is not allowed")
        }
        Ok(self
            .trick
            .take_back(id, &mut self.hands, self.propagated.throw_evaluation_policy)?)
    }

    pub fn finish_trick(&mut self) -> Result<Vec<MessageVariant>, Error> {
        if self.game_ended_early {
            bail!("Game has already ended; trick can't be finished");
        }
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
                                    let already_on_the_team =
                                        self.landlords_team.contains(&played.id);

                                    match self.propagated.multiple_join_policy {
                                        MultipleJoinPolicy::Unrestricted if already_on_the_team => {
                                            // double-join!
                                            friend.player_id = Some(played.id);
                                            msgs.push(MessageVariant::JoinedTeam {
                                                player: played.id,
                                                already_joined: true,
                                            });
                                        }
                                        MultipleJoinPolicy::NoDoubleJoin if already_on_the_team => {
                                        }
                                        MultipleJoinPolicy::Unrestricted
                                        | MultipleJoinPolicy::NoDoubleJoin => {
                                            friend.player_id = Some(played.id);
                                            self.landlords_team.push(played.id);
                                            msgs.push(MessageVariant::JoinedTeam {
                                                player: played.id,
                                                already_joined: false,
                                            });
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
            if self.propagated.should_reveal_kitty_at_end_of_game {
                msgs.push(MessageVariant::EndOfGameKittyReveal {
                    cards: self.kitty.clone(),
                });
            }
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

    pub fn compute_player_level_deltas<'a, 'b: 'a>(
        players: impl Iterator<Item = &'b mut Player>,
        non_landlord_level_bump: usize,
        landlord_level_bump: usize,
        landlords_team: &'a [PlayerID],
        landlord_won: bool,
        landlord: (PlayerID, Number),
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
                let initial_rank = player.rank();
                let landlord_successfully_defended_a: bool =
                    landlord.1 == Number::Ace && landlord_won;

                for bump_idx in 0..bump {
                    match advancement_policy {
                        AdvancementPolicy::FullyUnrestricted => (),

                        // Player *must* defend on Ace and win to advance.
                        _ if player.rank() == Number::Ace
                            && !(is_defending
                                && bump_idx == 0
                                && landlord_successfully_defended_a) =>
                        {
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
                        is_landlord: landlord.0 == player.id,
                        ranks_up: num_advances,
                        confetti: num_advances > 0
                            && landlord_won
                            && is_defending
                            && initial_rank == Number::Ace,
                    },
                )
            })
            .collect();

        msgs.push(MessageVariant::GameFinished { result });
        msgs
    }

    pub fn calculate_points(&self) -> (isize, isize) {
        let mut non_landlords_points: isize = self
            .points
            .iter()
            .filter(|(id, _)| !self.landlords_team.contains(id))
            .flat_map(|(_, cards)| cards)
            .flat_map(|c| c.points())
            .sum::<usize>() as isize;

        let observed_points = self
            .points
            .iter()
            .filter(|(id, _)| {
                !self.propagated.hide_landlord_points || !self.landlords_team.contains(id)
            })
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
        (non_landlords_points, observed_points)
    }

    pub fn game_finished(&self) -> bool {
        self.game_ended_early || self.hands.is_empty() && self.trick.played_cards().is_empty()
    }

    pub fn finish_game_early(&mut self) -> Result<MessageVariant, Error> {
        if self.game_finished() {
            bail!("Game has already ended");
        }
        let (non_landlords_points, observed_points) = self.calculate_points();
        let can_end_early = !next_threshold_reachable(
            &self.propagated.game_scoring_parameters,
            &self.decks,
            non_landlords_points,
            observed_points,
        )?;

        if can_end_early {
            self.game_ended_early = true;
            Ok(MessageVariant::GameEndedEarly)
        } else {
            bail!("Game can't be ended early; there are still points in play")
        }
    }

    pub fn finish_game(&self) -> Result<(InitializePhase, bool, Vec<MessageVariant>), Error> {
        let mut msgs = vec![];
        if !self.game_finished() {
            bail!("not done playing yet!")
        }

        let (non_landlords_points, _) = self.calculate_points();

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

        let mut propagated = self.propagated.clone();

        let GameScoreResult {
            non_landlord_delta: non_landlord_level_bump,
            landlord_delta: landlord_level_bump,
            landlord_won,
            landlord_bonus: bonus_level_earned,
        } = compute_level_deltas(
            &propagated.game_scoring_parameters,
            &self.decks,
            non_landlords_points,
            smaller_landlord_team,
        )?;

        msgs.push(MessageVariant::EndOfGameSummary {
            landlord_won,
            non_landlords_points,
        });

        if bonus_level_earned {
            msgs.push(MessageVariant::BonusLevelEarned);
        };

        let landlord_idx = bail_unwrap!(propagated
            .players
            .iter()
            .position(|p| p.id == self.landlord));

        msgs.extend(Self::compute_player_level_deltas(
            propagated.players.iter_mut(),
            non_landlord_level_bump,
            landlord_level_bump,
            &self.landlords_team[..],
            landlord_won,
            (self.landlord, self.propagated.players[landlord_idx].level),
            propagated.advancement_policy,
        ));

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

        Ok((InitializePhase { propagated }, landlord_won, msgs))
    }

    pub fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase { propagated }, msgs))
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
    #[serde(default)]
    removed_cards: Vec<Card>,
    #[serde(default)]
    decks: Vec<Deck>,
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

    pub fn num_friends(&self) -> usize {
        match self.game_mode {
            GameMode::FindingFriends { num_friends, .. } => num_friends,
            GameMode::Tractor => 0,
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
                if FriendSelectionPolicy::TrumpsIncluded != self.propagated.friend_selection_policy
                {
                    if friend.card.is_joker() || friend.card.number() == Some(self.trump.number()) {
                        bail!(
                            "you can't pick a joker or a {} as your friend",
                            self.trump.number().as_str()
                        )
                    }
                    if self.trump.suit() != None && friend.card.suit() == self.trump.suit() {
                        bail!("you can't pick a trump suit as your friend")
                    }
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

                if let FriendSelectionPolicy::PointCardNotAllowed =
                    self.propagated.friend_selection_policy
                {
                    let landlord = self.landlord;
                    let landlord_level = self
                        .propagated
                        .players
                        .iter()
                        .find(|p| p.id == landlord)
                        .ok_or_else(|| anyhow!("Couldn't find landlord level?"))?
                        .rank();

                    match (landlord_level, friend.card.points(), friend.card.number()) {
                        (Number::Ace, _, Some(Number::King)) => (),
                        (_, Some(_), _) => {
                            bail!("you can't pick a point card as your friend");
                        }
                        (_, _, _) => (),
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
        self.trump = match winning_bid.card {
            Card::Unknown => bail!("can't bid with unknown cards!"),
            Card::SmallJoker | Card::BigJoker => Trump::NoTrump {
                number: self.trump.number(),
            },
            Card::Suited { suit, .. } => Trump::Standard {
                suit,
                number: self.trump.number(),
            },
        };
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
            self.propagated.bid_reinforcement_policy,
            self.propagated.joker_bid_policy,
            self.num_decks,
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

    pub fn landlord(&self) -> PlayerID {
        self.landlord
    }

    pub fn hands(&self) -> &Hands {
        &self.hands
    }

    pub fn trump(&self) -> Trump {
        self.trump
    }

    pub fn next_player(&self) -> Result<PlayerID, Error> {
        if self.propagated.kitty_theft_policy == KittyTheftPolicy::AllowKittyTheft
            && self.autobid.is_none()
            && !self.finalized
        {
            Ok(self.exchanger.unwrap_or(self.landlord))
        } else {
            Ok(self.landlord)
        }
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
            game_ended_early: false,
            removed_cards: self.removed_cards.clone(),
            decks: self.decks.clone(),
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
    #[serde(default)]
    removed_cards: Vec<Card>,
    #[serde(default)]
    decks: Vec<Deck>,
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
            let (first_bid, winning_bid) = Bid::first_and_winner(&self.bids, self.autobid)?;
            let landlord = self.propagated.landlord.unwrap_or_else(|| {
                match self.propagated.first_landlord_selection_policy {
                    FirstLandlordSelectionPolicy::ByWinningBid => winning_bid.id,
                    FirstLandlordSelectionPolicy::ByFirstBid => first_bid.id,
                }
            });

            Ok(landlord)
        } else {
            Ok(self.propagated.players[self.position].id)
        }
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
            self.propagated.bid_reinforcement_policy,
            self.propagated.joker_bid_policy,
            self.num_decks,
            0,
        )
    }

    pub fn take_back_bid(&mut self, id: PlayerID) -> Result<(), Error> {
        Bid::take_back_bid(id, self.propagated.bid_takeback_policy, &mut self.bids, 0)
    }

    pub fn done_drawing(&self) -> bool {
        self.deck.is_empty()
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
                removed_cards: self.removed_cards.clone(),
                decks: self.decks.clone(),
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
            Some(level) if decks.iter().any(|d| d.includes_number(level)) => (),
            None if self
                .players
                .iter()
                .all(|p| decks.iter().any(|d| d.includes_number(p.level))) => {}
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
                    Some(level) if level == min_number => {
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
                            .map(|p| p.level)
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

#[cfg(test)]
mod tests {
    use super::{
        AdvancementPolicy, FriendSelection, GameMode, GameModeSettings, InitializePhase,
        KittyTheftPolicy, MessageVariant, PlayPhase, Player,
    };

    use crate::settings::FriendSelectionPolicy;
    use crate::types::{cards, Card, Number, PlayerID, FULL_DECK};

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
            (PlayerID(0), Number::Ace),
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
            (PlayerID(0), Number::Ace),
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
            (PlayerID(0), Number::Ace),
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
    fn test_unusual_kitty_sizes() {
        let mut init = InitializePhase::new();
        let p1 = init.add_player("p1".into()).unwrap().0;
        init.add_player("p2".into()).unwrap();
        init.add_player("p3".into()).unwrap();
        init.set_game_mode(GameModeSettings::FindingFriends { num_friends: None })
            .unwrap();
        for n_players in 4..10 {
            init.add_player(format!("p{}", n_players)).unwrap();
            for n_decks in 1..n_players {
                for kitty_size in 1..30 {
                    let mut init_ = init.clone();
                    init_.set_num_decks(Some(n_decks)).unwrap();
                    if init_.set_kitty_size(Some(kitty_size)).is_ok() {
                        let draw = init_.start(p1).unwrap();
                        assert_eq!(draw.deck.len() % n_players, 0);
                        assert_eq!(draw.kitty.len(), kitty_size);
                        assert_eq!(
                            draw.removed_cards.len() + draw.deck.len() + draw.kitty.len(),
                            n_decks * FULL_DECK.len()
                        );
                    }
                }
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
        let mut draw = init.start(PlayerID(0)).unwrap();
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
        let mut draw = init.start(PlayerID(0)).unwrap();
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
        let mut draw = init.start(PlayerID(0)).unwrap();

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
    fn test_set_friends() {
        use cards::*;

        let setup_exchange = |friend_selection_policy, bid: Card| {
            let mut init = InitializePhase::new();
            init.set_game_mode(GameModeSettings::FindingFriends { num_friends: None })
                .unwrap();
            init.set_friend_selection_policy(friend_selection_policy)
                .unwrap();
            let p1 = init.add_player("p1".into()).unwrap().0;
            let p2 = init.add_player("p2".into()).unwrap().0;
            let p3 = init.add_player("p3".into()).unwrap().0;
            let p4 = init.add_player("p4".into()).unwrap().0;
            init.set_landlord(Some(p2)).unwrap();
            init.set_rank(p2, bid.number().unwrap()).unwrap();

            let mut draw = init.start(PlayerID(1)).unwrap();
            draw.deck = vec![bid, bid, bid, bid];
            draw.draw_card(p2).unwrap();
            draw.draw_card(p3).unwrap();
            draw.draw_card(p4).unwrap();
            draw.draw_card(p1).unwrap();

            assert!(draw.bid(p1, bid, 1));

            (p2, draw.advance(p2).unwrap())
        };

        let test_cases = vec![
            (
                FriendSelectionPolicy::Unrestricted,
                S_7,
                vec![(C_K, true), (S_3, false), (C_3, true), (C_A, true)],
            ),
            (
                FriendSelectionPolicy::PointCardNotAllowed,
                S_7,
                vec![(C_K, false), (S_3, false), (C_3, true), (C_A, true)],
            ),
            (
                FriendSelectionPolicy::PointCardNotAllowed,
                S_A,
                vec![(C_K, true), (S_3, false), (C_3, true), (C_A, false)],
            ),
            (
                FriendSelectionPolicy::HighestCardNotAllowed,
                S_7,
                vec![(C_K, true), (S_3, false), (C_3, true), (C_A, false)],
            ),
            (
                FriendSelectionPolicy::TrumpsIncluded,
                S_7,
                vec![(C_K, true), (S_3, true), (C_3, true), (C_A, true)],
            ),
        ];

        for (friend_selection_policy, landlord_level, friends) in test_cases {
            for (friend, ok) in friends {
                let (p2, mut exchange) = setup_exchange(friend_selection_policy, landlord_level);

                assert_eq!(
                    exchange
                        .set_friends(
                            p2,
                            vec![FriendSelection {
                                card: friend,
                                initial_skip: 0,
                            }],
                        )
                        .is_ok(),
                    ok,
                    "Expected {:?} to be a {} friend for {:?}",
                    friend,
                    if ok { "legal" } else { "illegal" },
                    friend_selection_policy
                );
            }
        }
    }

    #[test]
    fn test_full_game_play() {
        use cards::*;

        let mut init = InitializePhase::new();

        init.set_game_mode(GameModeSettings::FindingFriends { num_friends: None })
            .unwrap();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        let p5 = init.add_player("p5".into()).unwrap().0;
        let p6 = init.add_player("p6".into()).unwrap().0;

        init.set_landlord(Some(p2)).unwrap();
        init.set_rank(p2, Number::Seven).unwrap();

        let mut draw = init.start(PlayerID(1)).unwrap();

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
        if let Ok((phase, _, _msgs)) = play.finish_game() {
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

        let mut draw = init.start(PlayerID(0)).unwrap();
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
                .filter(|m| matches!(m, MessageVariant::JoinedTeam { player, already_joined: false } if *player == p2))
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

        // We get a re-joined team message, since p2 has already joined.
        let msgs = play.finish_trick().unwrap();
        assert_eq!(
            msgs.into_iter()
                .filter(|m| matches!(m, MessageVariant::JoinedTeam { player, already_joined: true } if *player == p2))
                .count(),
            1
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

        let (new_init_phase, _, msgs) = play.finish_game().unwrap();
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
