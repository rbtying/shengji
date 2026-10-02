use std::collections::HashSet;

use anyhow::{anyhow, bail, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shengji_mechanics::bidding::Bid;
use shengji_mechanics::deck::Deck;
use shengji_mechanics::hands::Hands;
use shengji_mechanics::types::{Card, EffectiveSuit, Number, PlayerID, Rank, Trump};

use crate::message::MessageVariant;
use crate::settings::{
    Friend, FriendSelection, FriendSelectionPolicy, GameMode, KittyTheftPolicy, PropagatedState,
};

use crate::game_state::{initialize_phase::InitializePhase, play_phase::PlayPhase};

macro_rules! bail_unwrap {
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => return Err(anyhow!("option was none")),
        }
    };
}

const RIVER_CROSSING_CARD_COUNT: usize = 5;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum RiverCrossingStage {
    Deciding,
    SelectingCrossingCards,
    SelectingReturnCards,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiverCrossingPlayerState {
    player_id: PlayerID,
    eligible: bool,
    decision: Option<bool>,
    crossing_cards: Vec<Card>,
    crossing_cards_submitted: bool,
    received_crossing_cards: Vec<Card>,
    return_cards: Vec<Card>,
    return_cards_submitted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiverCrossingState {
    stage: RiverCrossingStage,
    players: Vec<RiverCrossingPlayerState>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub enum KittyTheftStage {
    #[default]
    Disabled,
    Exchanging,
    Waiting,
    Complete,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, JsonSchema, Default)]
pub struct KittyTheftState {
    stage: KittyTheftStage,
    current_player: Option<PlayerID>,
    remaining_players: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExchangePhase {
    propagated: PropagatedState,
    num_decks: usize,
    game_mode: GameMode,
    hands: Hands,
    kitty: Vec<Card>,
    kitty_size: usize,
    landlord: PlayerID,
    trump: Trump,
    exchanger: PlayerID,
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
    #[serde(default)]
    river_crossing: Option<RiverCrossingState>,
    #[serde(default)]
    kitty_theft: KittyTheftState,
    player_requested_reset: Option<PlayerID>,
}

impl ExchangePhase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        propagated: PropagatedState,
        num_decks: usize,
        game_mode: GameMode,
        kitty: Vec<Card>,
        landlord: PlayerID,
        hands: Hands,
        trump: Trump,
        bids: Vec<Bid>,
        autobid: Option<Bid>,
        removed_cards: Vec<Card>,
        decks: Vec<Deck>,
    ) -> Self {
        let kitty_theft = if propagated.kitty_theft_policy == KittyTheftPolicy::AllowKittyTheft
            && autobid.is_none()
        {
            KittyTheftState {
                stage: KittyTheftStage::Exchanging,
                ..KittyTheftState::default()
            }
        } else {
            KittyTheftState::default()
        };
        ExchangePhase {
            kitty_size: kitty.len(),
            num_decks,
            game_mode,
            kitty,
            propagated,
            landlord,
            exchanger: landlord,
            hands,
            trump,
            bids,
            autobid,
            removed_cards,
            decks,
            river_crossing: None,
            kitty_theft,
            finalized: false,
            epoch: 1,
            player_requested_reset: None,
        }
    }

    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

    pub fn move_card_to_kitty(&mut self, id: PlayerID, card: Card) -> Result<(), Error> {
        if self.river_crossing.is_some() {
            bail!("bottom-card exchange has already ended")
        }
        if self.exchanger != id {
            bail!("not the exchanger")
        }
        if self.finalized {
            bail!("cards already finalized")
        }
        self.hands.remove(self.exchanger, Some(card))?;
        self.kitty.push(card);
        Ok(())
    }

    pub fn move_card_to_hand(&mut self, id: PlayerID, card: Card) -> Result<(), Error> {
        if self.river_crossing.is_some() {
            bail!("bottom-card exchange has already ended")
        }
        if self.exchanger != id {
            bail!("not the exchanger")
        }
        if self.finalized {
            bail!("cards already finalized")
        }
        if let Some(index) = self.kitty.iter().position(|c| *c == card) {
            self.kitty.swap_remove(index);
            self.hands.add(self.exchanger, Some(card))?;
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
        if self.river_crossing.is_some() {
            bail!("friend selection has already ended")
        }
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
                    if friend.card.is_joker() || friend.card.number() == self.trump.number() {
                        if let Some(n) = self.trump.number() {
                            bail!("you can't pick a joker or a {} as your friend", n.as_str())
                        } else {
                            bail!("you can't pick a joker as your friend",)
                        }
                    }
                    if self.trump.suit().is_some() && friend.card.suit() == self.trump.suit() {
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
                        (Some(Number::Ace), Some(Number::King)) | (_, Some(Number::Ace)) => {
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
                        (Rank::Number(Number::Ace), _, Some(Number::King)) => (),
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
        if self.river_crossing.is_some() {
            bail!("bottom-card exchange has already ended")
        }
        if id != self.exchanger {
            bail!("only the exchanger can finalize their cards")
        }
        if self.finalized {
            bail!("Already finalized")
        }
        if self.kitty.len() != self.kitty_size {
            bail!("incorrect number of cards in the bottom")
        }
        self.finalized = true;
        if self.kitty_theft.stage == KittyTheftStage::Exchanging {
            if self.winning_bid_is_unbeatable() {
                self.kitty_theft = KittyTheftState {
                    stage: KittyTheftStage::Complete,
                    ..KittyTheftState::default()
                };
            } else {
                self.start_kitty_theft_round()?;
            }
        }
        Ok(())
    }

    fn winning_bid_is_unbeatable(&self) -> bool {
        self.bids.last().is_some_and(|bid| {
            bid.card == Card::BigJoker && bid.count == self.num_decks
        })
    }

    fn player_after(&self, id: PlayerID) -> Result<PlayerID, Error> {
        let position = self
            .propagated
            .players
            .iter()
            .position(|player| player.id == id)
            .ok_or_else(|| anyhow!("player not found"))?;
        let next = (position + 1) % self.propagated.players.len();
        Ok(self.propagated.players[next].id)
    }

    fn start_kitty_theft_round(&mut self) -> Result<(), Error> {
        let remaining_players = self.propagated.players.len().saturating_sub(1);
        if remaining_players == 0 {
            self.kitty_theft = KittyTheftState {
                stage: KittyTheftStage::Complete,
                ..KittyTheftState::default()
            };
        } else {
            self.kitty_theft = KittyTheftState {
                stage: KittyTheftStage::Waiting,
                current_player: Some(self.player_after(self.exchanger)?),
                remaining_players,
            };
        }
        Ok(())
    }

    pub fn decline_kitty_theft(&mut self, id: PlayerID) -> Result<(), Error> {
        if self.kitty_theft.stage != KittyTheftStage::Waiting
            || self.kitty_theft.current_player != Some(id)
        {
            bail!("not this player's turn to decide whether to steal the bottom")
        }
        if self.kitty_theft.remaining_players <= 1 {
            self.kitty_theft = KittyTheftState {
                stage: KittyTheftStage::Complete,
                ..KittyTheftState::default()
            };
        } else {
            self.kitty_theft.current_player = Some(self.player_after(id)?);
            self.kitty_theft.remaining_players -= 1;
        }
        Ok(())
    }

    pub fn pick_up_cards(&mut self, id: PlayerID) -> Result<(), Error> {
        if self.river_crossing.is_some() {
            bail!("bottom-card exchange has already ended")
        }
        if !self.finalized {
            bail!("Current exchanger is still exchanging cards!")
        }
        if self.autobid.is_some() {
            bail!("Bid was automatically determined; no overbidding allowed")
        }
        if self.kitty_theft.stage != KittyTheftStage::Waiting
            || self.kitty_theft.current_player != Some(id)
        {
            bail!("not this player's turn to steal the bottom")
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
                number: self
                    .trump
                    .number()
                    .expect("Shouldn't have trump number if there are bids"),
            },
        };
        self.finalized = false;
        self.epoch += 1;
        self.exchanger = winning_bid.id;
        self.kitty_theft = KittyTheftState {
            stage: KittyTheftStage::Exchanging,
            ..KittyTheftState::default()
        };

        Ok(())
    }

    pub fn bid(&mut self, id: PlayerID, card: Card, count: usize) -> Result<bool, Error> {
        if self.river_crossing.is_some()
            || !self.finalized
            || self.autobid.is_some()
            || self.kitty_theft.stage != KittyTheftStage::Waiting
            || self.kitty_theft.current_player != Some(id)
        {
            return Ok(false);
        }
        if !Bid::bid(
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
        ) {
            return Ok(false);
        }
        self.pick_up_cards(id)?;
        Ok(true)
    }

    pub fn take_back_bid(&mut self, id: PlayerID) -> Result<(), Error> {
        if self.river_crossing.is_some() {
            bail!("bidding has already ended")
        }
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

    pub fn propagated(&self) -> &PropagatedState {
        &self.propagated
    }

    pub fn propagated_mut(&mut self) -> &mut PropagatedState {
        &mut self.propagated
    }

    pub fn next_player(&self) -> Result<PlayerID, Error> {
        if self.river_crossing.is_some() {
            // River-crossing choices are simultaneous, so there is deliberately
            // no public "next" player. Returning the landlord also avoids
            // revealing which players are eligible or have already submitted.
            return Ok(self.landlord);
        }
        match self.kitty_theft.stage {
            KittyTheftStage::Exchanging => Ok(self.exchanger),
            KittyTheftStage::Waiting => self
                .kitty_theft
                .current_player
                .ok_or_else(|| anyhow!("missing current player for kitty-theft decision")),
            KittyTheftStage::Disabled | KittyTheftStage::Complete => Ok(self.landlord),
        }
    }

    fn kitty_theft_complete(&self) -> bool {
        matches!(
            self.kitty_theft.stage,
            KittyTheftStage::Disabled | KittyTheftStage::Complete
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

        if !self.kitty_theft_complete() {
            bail!("must finish sequential kitty-theft decisions first")
        }

        if self.river_crossing_is_applicable() {
            match self.river_crossing.as_ref().map(|r| r.stage) {
                Some(RiverCrossingStage::Complete) => (),
                _ => bail!("must complete five-trump river crossing first"),
            }
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

        PlayPhase::new(
            self.propagated.clone(),
            self.num_decks,
            self.game_mode.clone(),
            self.hands.clone(),
            self.kitty.clone(),
            self.trump,
            self.landlord,
            self.exchanger,
            landlords_team,
            self.removed_cards.clone(),
            self.decks.clone(),
            self.river_crossing_initiators(),
        )
    }

    fn river_crossing_initiators(&self) -> Vec<PlayerID> {
        self.river_crossing
            .as_ref()
            .filter(|r| r.stage != RiverCrossingStage::Deciding)
            .map(|r| {
                r.players
                    .iter()
                    .filter(|p| p.decision == Some(true))
                    .map(|p| p.player_id)
                    .collect()
            })
            .unwrap_or_default()
    }

    fn river_crossing_is_applicable(&self) -> bool {
        self.propagated.five_trump_river_crossing_enabled
            && self.propagated.players.len() == 4
            && matches!(self.game_mode, GameMode::Tractor)
            && matches!(self.trump, Trump::Standard { .. })
    }

    fn partner_of(&self, player_id: PlayerID) -> Result<PlayerID, Error> {
        let position = self
            .propagated
            .players
            .iter()
            .position(|p| p.id == player_id)
            .ok_or_else(|| anyhow!("player not found"))?;
        if self.propagated.players.len() != 4 {
            bail!("five-trump river crossing requires exactly four players")
        }
        Ok(self.propagated.players[(position + 2) % 4].id)
    }

    fn partner_decision(&self, player_id: PlayerID) -> Option<bool> {
        let partner = self.partner_of(player_id).ok()?;
        self.river_crossing
            .as_ref()?
            .players
            .iter()
            .find(|p| p.player_id == partner)?
            .decision
    }

    fn trump_count(&self, player_id: PlayerID) -> Result<usize, Error> {
        Ok(self
            .hands
            .get(player_id)?
            .iter()
            .filter(|(card, _)| self.trump.effective_suit(**card) == EffectiveSuit::Trump)
            .map(|(_, count)| *count)
            .sum())
    }

    pub fn start_river_crossing(&mut self, id: PlayerID) -> Result<bool, Error> {
        if id != self.landlord {
            bail!("only the leader can advance the game")
        }
        if !self.river_crossing_is_applicable() {
            return Ok(false);
        }
        if self.river_crossing.is_some() {
            bail!("five-trump river crossing has already started")
        }

        // Reuse all pre-play checks, except the crossing-completion check.
        if self.kitty.len() != self.kitty_size {
            bail!("incorrect number of cards in the bottom")
        }
        if !self.kitty_theft_complete() {
            bail!("must finish sequential kitty-theft decisions first")
        }

        let mut players = Vec::with_capacity(4);
        for player in &self.propagated.players {
            let eligible = self.trump_count(player.id)? <= RIVER_CROSSING_CARD_COUNT;
            players.push(RiverCrossingPlayerState {
                player_id: player.id,
                eligible,
                decision: if eligible { None } else { Some(false) },
                crossing_cards: vec![],
                crossing_cards_submitted: false,
                received_crossing_cards: vec![],
                return_cards: vec![],
                return_cards_submitted: false,
            });
        }
        let stage = if players.iter().any(|p| p.eligible) {
            RiverCrossingStage::Deciding
        } else {
            RiverCrossingStage::Complete
        };
        self.river_crossing = Some(RiverCrossingState { stage, players });
        Ok(true)
    }

    pub fn decide_river_crossing(
        &mut self,
        id: PlayerID,
        cross: bool,
    ) -> Result<Vec<MessageVariant>, Error> {
        let crossing = self
            .river_crossing
            .as_mut()
            .ok_or_else(|| anyhow!("five-trump river crossing has not started"))?;
        if crossing.stage != RiverCrossingStage::Deciding {
            bail!("river-crossing decisions are already complete")
        }
        let player = crossing
            .players
            .iter_mut()
            .find(|p| p.player_id == id)
            .ok_or_else(|| anyhow!("player not found"))?;
        if !player.eligible {
            bail!("player has more than five trump cards")
        }
        if player.decision.is_some() {
            bail!("player has already decided")
        }
        player.decision = Some(cross);

        if crossing.players.iter().all(|p| p.decision.is_some()) {
            crossing.stage = if crossing.players.iter().any(|p| p.decision == Some(true)) {
                RiverCrossingStage::SelectingCrossingCards
            } else {
                RiverCrossingStage::Complete
            };
        }
        // Announce only after all simultaneous decisions are locked in. The
        // last responder may have declined, so name each initiator explicitly.
        Ok(self
            .river_crossing_initiators()
            .into_iter()
            .map(|player| MessageVariant::RiverCrossingInitiated { player })
            .collect())
    }

    fn validate_crossing_cards(&self, id: PlayerID, cards: &[Card]) -> Result<(), Error> {
        if cards.len() != RIVER_CROSSING_CARD_COUNT {
            bail!("must select exactly five cards")
        }
        self.hands.contains(id, cards.iter().copied())?;
        let selected_trumps = cards
            .iter()
            .filter(|card| self.trump.effective_suit(**card) == EffectiveSuit::Trump)
            .count();
        if selected_trumps != self.trump_count(id)? {
            bail!("the five crossing cards must include every trump card in the hand")
        }
        Ok(())
    }

    pub fn submit_river_crossing_cards(
        &mut self,
        id: PlayerID,
        cards: Vec<Card>,
    ) -> Result<(), Error> {
        let stage = self
            .river_crossing
            .as_ref()
            .map(|r| r.stage)
            .ok_or_else(|| anyhow!("five-trump river crossing has not started"))?;
        if stage != RiverCrossingStage::SelectingCrossingCards {
            bail!("not selecting river-crossing cards")
        }
        self.validate_crossing_cards(id, &cards)?;
        {
            let player = self
                .river_crossing
                .as_mut()
                .unwrap()
                .players
                .iter_mut()
                .find(|p| p.player_id == id)
                .ok_or_else(|| anyhow!("player not found"))?;
            if player.decision != Some(true) {
                bail!("player did not choose to cross the river")
            }
            if player.crossing_cards_submitted {
                bail!("crossing cards already submitted")
            }
            player.crossing_cards = cards;
            player.crossing_cards_submitted = true;
        }

        let all_submitted = self
            .river_crossing
            .as_ref()
            .unwrap()
            .players
            .iter()
            .all(|p| p.decision != Some(true) || p.crossing_cards_submitted);
        if all_submitted {
            self.apply_crossing_cards()?;
        }
        Ok(())
    }

    fn apply_crossing_cards(&mut self) -> Result<(), Error> {
        let transfers = self
            .river_crossing
            .as_ref()
            .unwrap()
            .players
            .iter()
            .filter(|p| p.decision == Some(true))
            .map(|p| {
                Ok((
                    p.player_id,
                    self.partner_of(p.player_id)?,
                    p.crossing_cards.clone(),
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        for (from, _, cards) in &transfers {
            self.hands.remove(*from, cards.iter().copied())?;
        }
        for (_, to, cards) in &transfers {
            self.hands.add(*to, cards.iter().copied())?;
            let recipient = self
                .river_crossing
                .as_mut()
                .unwrap()
                .players
                .iter_mut()
                .find(|p| p.player_id == *to)
                .unwrap();
            recipient.received_crossing_cards = cards.clone();
        }
        self.river_crossing.as_mut().unwrap().stage = RiverCrossingStage::SelectingReturnCards;
        Ok(())
    }

    pub fn submit_river_return_cards(
        &mut self,
        id: PlayerID,
        cards: Vec<Card>,
    ) -> Result<(), Error> {
        let crossing = self
            .river_crossing
            .as_ref()
            .ok_or_else(|| anyhow!("five-trump river crossing has not started"))?;
        if crossing.stage != RiverCrossingStage::SelectingReturnCards {
            bail!("not selecting return cards")
        }
        if self.partner_decision(id) != Some(true) {
            bail!("this player has no cards to return")
        }
        if cards.len() != RIVER_CROSSING_CARD_COUNT {
            bail!("must return exactly five cards")
        }
        self.hands.contains(id, cards.iter().copied())?;
        {
            let player = self
                .river_crossing
                .as_mut()
                .unwrap()
                .players
                .iter_mut()
                .find(|p| p.player_id == id)
                .ok_or_else(|| anyhow!("player not found"))?;
            if player.return_cards_submitted {
                bail!("return cards already submitted")
            }
            player.return_cards = cards;
            player.return_cards_submitted = true;
        }

        let all_submitted = self
            .river_crossing
            .as_ref()
            .unwrap()
            .players
            .iter()
            .all(|p| self.partner_decision(p.player_id) != Some(true) || p.return_cards_submitted);
        if all_submitted {
            self.apply_return_cards()?;
        }
        Ok(())
    }

    fn apply_return_cards(&mut self) -> Result<(), Error> {
        let transfers = self
            .river_crossing
            .as_ref()
            .unwrap()
            .players
            .iter()
            .filter(|p| p.return_cards_submitted)
            .map(|p| {
                Ok((
                    p.player_id,
                    self.partner_of(p.player_id)?,
                    p.return_cards.clone(),
                ))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        // Remove every selection before adding any returned cards. This makes a
        // two-way return genuinely simultaneous even when identical cards occur.
        for (from, _, cards) in &transfers {
            self.hands.remove(*from, cards.iter().copied())?;
        }
        for (_, to, cards) in &transfers {
            self.hands.add(*to, cards.iter().copied())?;
        }
        self.river_crossing.as_mut().unwrap().stage = RiverCrossingStage::Complete;
        Ok(())
    }

    pub fn river_crossing_complete(&self) -> bool {
        self.river_crossing
            .as_ref()
            .map(|r| r.stage == RiverCrossingStage::Complete)
            .unwrap_or(false)
    }

    pub fn request_reset(
        &mut self,
        player: PlayerID,
    ) -> Result<(Option<InitializePhase>, Vec<MessageVariant>), Error> {
        match self.player_requested_reset {
            Some(p) => {
                // ignore duplicate reset requests from same player
                if p == player {
                    return Ok((None, vec![]));
                }

                let (s, m) = self.return_to_initialize()?;
                Ok((Some(s), m))
            }
            None => {
                self.player_requested_reset = Some(player);
                Ok((None, vec![MessageVariant::ResetRequested]))
            }
        }
    }

    pub fn cancel_reset(&mut self) -> Option<MessageVariant> {
        if self.player_requested_reset.is_some() {
            self.player_requested_reset = None;
            return Some(MessageVariant::ResetCanceled);
        }
        None
    }

    fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase::from_propagated(propagated), msgs))
    }

    pub fn destructively_redact_for_player(&mut self, player: PlayerID) {
        let players_partner = self.partner_of(player).ok();
        self.hands.destructively_redact_except_for_player(player);
        if player != self.exchanger || self.finalized {
            self.kitty.fill(Card::Unknown);
        }
        if player != self.landlord {
            if let GameMode::FindingFriends {
                ref mut friends, ..
            } = self.game_mode
            {
                friends.clear();
            }
        }
        if let Some(ref mut crossing) = self.river_crossing {
            let stage = crossing.stage;
            for state in &mut crossing.players {
                if state.player_id != player {
                    if stage == RiverCrossingStage::Deciding {
                        state.eligible = false;
                        state.decision = None;
                    }
                    state.crossing_cards.clear();
                    state.return_cards.clear();
                    if players_partner != Some(state.player_id) {
                        state.received_crossing_cards.clear();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod river_crossing_tests {
    use super::*;
    use shengji_mechanics::player::Player;
    use shengji_mechanics::types::{
        cards::{C_4, C_8, D_2, D_5, D_9, H_3, H_4, H_5, S_10, S_2, S_3, S_6, S_7},
        Rank, Suit,
    };

    const P1: PlayerID = PlayerID(1);
    const P2: PlayerID = PlayerID(2);
    const P3: PlayerID = PlayerID(3);
    const P4: PlayerID = PlayerID(4);

    fn player(id: PlayerID, name: &str) -> Player {
        Player {
            id,
            name: name.to_string(),
            level: Rank::Number(Number::Two),
            metalevel: 0,
        }
    }

    fn phase(p1_cards: Vec<Card>, p3_cards: Vec<Card>) -> ExchangePhase {
        let mut propagated = PropagatedState::default();
        propagated.players = vec![
            player(P1, "p1"),
            player(P2, "p2"),
            player(P3, "p3"),
            player(P4, "p4"),
        ];
        propagated.five_trump_river_crossing_enabled = true;
        propagated.game_mode = crate::settings::GameModeSettings::Tractor;

        let trump = Trump::Standard {
            suit: Suit::Hearts,
            number: Number::Two,
        };
        let mut hands = Hands::new([P1, P2, P3, P4]);
        hands.set_trump(trump);
        hands.add(P1, p1_cards).unwrap();
        hands.add(P2, vec![H_5; 6]).unwrap();
        hands.add(P3, p3_cards).unwrap();
        hands.add(P4, vec![H_5; 6]).unwrap();

        ExchangePhase::new(
            propagated,
            2,
            GameMode::Tractor,
            vec![],
            P1,
            hands,
            trump,
            vec![],
            None,
            vec![],
            vec![],
        )
    }

    fn enable_sequential_kitty_theft(phase: &mut ExchangePhase) {
        phase.propagated.kitty_theft_policy = KittyTheftPolicy::AllowKittyTheft;
        phase.kitty_theft = KittyTheftState {
            stage: KittyTheftStage::Exchanging,
            ..KittyTheftState::default()
        };
    }

    #[test]
    fn sequential_kitty_theft_must_complete_before_river_crossing() {
        let mut phase = phase(
            vec![S_3, C_4, D_5, S_6, S_7],
            vec![H_4, S_7, C_8, D_9, S_10],
        );
        enable_sequential_kitty_theft(&mut phase);

        phase.finalize(P1).unwrap();
        assert_eq!(phase.next_player().unwrap(), P2);
        assert!(phase.start_river_crossing(P1).is_err());
        assert!(phase.advance(P1).is_err());

        phase.decline_kitty_theft(P2).unwrap();
        assert_eq!(phase.next_player().unwrap(), P3);
        phase.decline_kitty_theft(P3).unwrap();
        assert_eq!(phase.next_player().unwrap(), P4);
        phase.decline_kitty_theft(P4).unwrap();

        assert_eq!(phase.next_player().unwrap(), P1);
        assert!(phase.start_river_crossing(P1).unwrap());
    }

    #[test]
    fn sequential_kitty_theft_turn_is_public_and_survives_serialization() {
        let mut phase = phase(
            vec![S_3, C_4, D_5, S_6, S_7],
            vec![H_4, S_7, C_8, D_9, S_10],
        );
        enable_sequential_kitty_theft(&mut phase);
        phase.finalize(P1).unwrap();

        let serialized = serde_json::to_value(&phase).unwrap();
        assert_eq!(serialized["kitty_theft"]["stage"], "Waiting");
        assert_eq!(serialized["kitty_theft"]["current_player"], 2);
        assert_eq!(serialized["kitty_theft"]["remaining_players"], 3);

        let restored: ExchangePhase = serde_json::from_value(serialized).unwrap();
        assert_eq!(restored.next_player().unwrap(), P2);
        for viewer in [P1, P2, P3, P4, PlayerID(99)] {
            let mut visible = restored.clone();
            visible.destructively_redact_for_player(viewer);
            let visible = serde_json::to_value(visible).unwrap();
            assert_eq!(visible["kitty_theft"]["current_player"], 2);
            assert_eq!(visible["kitty_theft"]["remaining_players"], 3);
        }
    }

    #[test]
    fn announcements_wait_for_all_decisions_and_name_the_initiator() {
        use crate::game_state::GameState;
        use crate::interactive::{Action, InteractiveGame};
        use slog::{o, Logger};

        let phase = phase(
            vec![S_3, C_4, D_5, S_6, S_7],
            vec![H_4, S_7, C_8, D_9, S_10],
        );
        let mut game = InteractiveGame::new_from_state(GameState::Exchange(phase));
        let logger = Logger::root(slog::Discard, o!());
        game.interact(Action::BeginPlay, P1, &logger).unwrap();
        assert!(game
            .interact(Action::DecideRiverCrossing(true), P1, &logger)
            .unwrap()
            .is_empty());
        let msgs = game
            .interact(Action::DecideRiverCrossing(false), P3, &logger)
            .unwrap();
        assert_eq!(msgs.len(), 1);
        let broadcast = serde_json::to_value(&msgs[0].0).unwrap();
        assert_eq!(
            broadcast["variant"],
            serde_json::json!({
                "type": "RiverCrossingInitiated", "player": 1
            })
        );
        assert!(msgs[0].1.starts_with("p1 发起了五主过河"));
        // The final responder is p3; the public message must still name p1.
        assert_eq!(broadcast["actor"], serde_json::json!(3));
        assert!(game
            .interact(Action::DecideRiverCrossing(true), P1, &logger)
            .is_err());
    }

    #[test]
    fn multiple_initiators_are_announced_once_each() {
        let mut phase = phase(
            vec![S_3, C_4, D_5, S_6, S_7],
            vec![H_4, S_7, C_8, D_9, S_10],
        );
        phase.start_river_crossing(P1).unwrap();
        assert!(phase.decide_river_crossing(P1, true).unwrap().is_empty());
        let msgs = phase.decide_river_crossing(P3, true).unwrap();
        assert_eq!(msgs.len(), 2);
        assert!(matches!(
            msgs[0],
            MessageVariant::RiverCrossingInitiated { player: P1 }
        ));
        assert!(matches!(
            msgs[1],
            MessageVariant::RiverCrossingInitiated { player: P3 }
        ));
        assert!(phase.decide_river_crossing(P3, true).is_err());
    }

    #[test]
    fn declined_and_disabled_crossings_have_no_markers() {
        let mut phase = phase(
            vec![S_3, C_4, D_5, S_6, S_7],
            vec![H_4, S_7, C_8, D_9, S_10],
        );
        let mut disabled = phase.clone();
        disabled.propagated.five_trump_river_crossing_enabled = false;
        assert!(!disabled.start_river_crossing(P1).unwrap());
        let play = serde_json::to_value(disabled.advance(P1).unwrap()).unwrap();
        assert_eq!(play["river_crossing_initiators"], serde_json::json!([]));

        phase.start_river_crossing(P1).unwrap();
        assert!(phase.decide_river_crossing(P1, false).unwrap().is_empty());
        assert!(phase.decide_river_crossing(P3, false).unwrap().is_empty());
        let play = serde_json::to_value(phase.advance(P1).unwrap()).unwrap();
        assert_eq!(play["river_crossing_initiators"], serde_json::json!([]));
    }

    #[test]
    fn initiators_survive_play_serialization_and_clear_on_reset() {
        let cards = vec![S_3, C_4, D_5, S_6, S_7];
        let mut phase = phase(cards.clone(), vec![H_4, S_7, C_8, D_9, S_10]);
        phase.start_river_crossing(P1).unwrap();
        phase.decide_river_crossing(P1, true).unwrap();
        phase.decide_river_crossing(P3, false).unwrap();
        phase
            .submit_river_crossing_cards(P1, cards.clone())
            .unwrap();
        phase.submit_river_return_cards(P3, cards).unwrap();
        let play = phase.advance(P1).unwrap();
        let serialized = serde_json::to_value(&play).unwrap();
        assert_eq!(
            serialized["river_crossing_initiators"],
            serde_json::json!([1])
        );
        let mut restored: PlayPhase = serde_json::from_value(serialized.clone()).unwrap();
        // Finishing the deal also drops its marker instead of propagating it.
        let mut finished = serialized.clone();
        finished["game_ended_early"] = serde_json::json!(true);
        finished["decks"] = serde_json::json!(vec![Deck::default(); 2]);
        let finished: PlayPhase = serde_json::from_value(finished).unwrap();
        let (next, _, _) = finished.finish_game().unwrap();
        assert!(!serde_json::to_string(&next)
            .unwrap()
            .contains("river_crossing_initiators"));
        // Every player and a late spectator receive the same public marker.
        for viewer in [P1, P2, P3, P4, PlayerID(99)] {
            let mut visible = restored.clone();
            visible.destructively_redact_for_player(viewer);
            assert_eq!(
                serde_json::to_value(visible).unwrap()["river_crossing_initiators"],
                serde_json::json!([1])
            );
        }
        // A state saved before this feature remains readable.
        let mut legacy = serialized;
        legacy
            .as_object_mut()
            .unwrap()
            .remove("river_crossing_initiators");
        let legacy: PlayPhase = serde_json::from_value(legacy).unwrap();
        assert_eq!(
            serde_json::to_value(legacy).unwrap()["river_crossing_initiators"],
            serde_json::json!([])
        );

        assert!(restored.request_reset(P1).unwrap().0.is_none());
        let init = restored.request_reset(P2).unwrap().0.unwrap();
        assert!(!serde_json::to_string(&init)
            .unwrap()
            .contains("river_crossing_initiators"));
        let draw = init.start(P1).unwrap();
        assert!(!serde_json::to_string(&draw)
            .unwrap()
            .contains("river_crossing_initiators"));
    }

    #[test]
    fn zero_trumps_is_eligible() {
        let p1_cards = vec![S_3, C_4, D_5, S_6, S_7];
        let mut phase = phase(p1_cards, vec![H_4, S_7, C_8, D_9, S_10]);
        assert!(phase.start_river_crossing(P1).unwrap());
        let p1 = phase
            .river_crossing
            .as_ref()
            .unwrap()
            .players
            .iter()
            .find(|p| p.player_id == P1)
            .unwrap();
        assert!(p1.eligible);
    }

    #[test]
    fn jokers_and_off_suit_level_cards_count_as_trumps() {
        let mut phase = phase(
            vec![Card::SmallJoker, Card::BigJoker, S_2, D_2, H_3, H_4, C_4],
            vec![H_4, S_7, C_8, D_9, S_10],
        );
        phase.start_river_crossing(P1).unwrap();
        let p1 = phase
            .river_crossing
            .as_ref()
            .unwrap()
            .players
            .iter()
            .find(|p| p.player_id == P1)
            .unwrap();
        assert!(!p1.eligible);
        assert_eq!(p1.decision, Some(false));
    }

    #[test]
    fn crossing_cards_must_include_every_trump() {
        let mut phase = phase(
            vec![H_3, S_3, C_4, D_5, S_6, S_7],
            vec![H_4, S_7, C_8, D_9, S_10],
        );
        phase.start_river_crossing(P1).unwrap();
        phase.decide_river_crossing(P1, true).unwrap();
        phase.decide_river_crossing(P3, false).unwrap();
        assert!(phase
            .submit_river_crossing_cards(P1, vec![S_3, C_4, D_5, S_6, S_7])
            .is_err());
    }

    #[test]
    fn two_way_crossing_has_a_simultaneous_return_round() {
        let p1_cards = vec![H_3, S_3, C_4, D_5, S_6];
        let p3_cards = vec![H_4, S_7, C_8, D_9, S_10];
        let mut phase = phase(p1_cards.clone(), p3_cards.clone());
        phase.start_river_crossing(P1).unwrap();
        phase.decide_river_crossing(P1, true).unwrap();
        phase.decide_river_crossing(P3, true).unwrap();

        phase
            .submit_river_crossing_cards(P1, p1_cards.clone())
            .unwrap();
        assert_eq!(
            phase.river_crossing.as_ref().unwrap().stage,
            RiverCrossingStage::SelectingCrossingCards
        );
        phase
            .submit_river_crossing_cards(P3, p3_cards.clone())
            .unwrap();
        assert_eq!(
            phase.river_crossing.as_ref().unwrap().stage,
            RiverCrossingStage::SelectingReturnCards
        );
        assert!(phase.hands.contains(P1, p3_cards.clone()).is_ok());
        assert!(phase.hands.contains(P3, p1_cards.clone()).is_ok());

        phase
            .submit_river_return_cards(P1, p3_cards.clone())
            .unwrap();
        assert!(!phase.river_crossing_complete());
        // P3's selection remains secret and neither return is applied early.
        assert!(phase.hands.contains(P1, p3_cards.clone()).is_ok());
        phase
            .submit_river_return_cards(P3, p1_cards.clone())
            .unwrap();
        assert!(phase.river_crossing_complete());
        assert!(phase.hands.contains(P1, p1_cards).is_ok());
        assert!(phase.hands.contains(P3, p3_cards).is_ok());
    }

    #[test]
    fn one_way_recipient_can_return_received_cards() {
        let p1_cards = vec![H_3, S_3, C_4, D_5, S_6];
        let p3_cards = vec![H_4, S_7, C_8, D_9, S_10];
        let mut phase = phase(p1_cards.clone(), p3_cards);
        phase.start_river_crossing(P1).unwrap();
        phase.decide_river_crossing(P1, true).unwrap();
        phase.decide_river_crossing(P3, false).unwrap();
        phase
            .submit_river_crossing_cards(P1, p1_cards.clone())
            .unwrap();
        phase
            .submit_river_return_cards(P3, p1_cards.clone())
            .unwrap();
        assert!(phase.river_crossing_complete());
        assert!(phase.hands.contains(P1, p1_cards).is_ok());
    }

    #[test]
    fn private_decisions_and_card_selections_are_redacted() {
        let p1_cards = vec![H_3, S_3, C_4, D_5, S_6];
        let p3_cards = vec![H_4, S_7, C_8, D_9, S_10];
        let mut phase = phase(p1_cards.clone(), p3_cards.clone());
        phase.start_river_crossing(P1).unwrap();
        phase.decide_river_crossing(P1, true).unwrap();

        let mut p3_view = phase.clone();
        p3_view.destructively_redact_for_player(P3);
        let hidden_p1 = p3_view
            .river_crossing
            .as_ref()
            .unwrap()
            .players
            .iter()
            .find(|p| p.player_id == P1)
            .unwrap();
        assert_eq!(hidden_p1.decision, None);

        phase.decide_river_crossing(P3, true).unwrap();
        phase
            .submit_river_crossing_cards(P1, p1_cards.clone())
            .unwrap();
        let mut p3_view = phase.clone();
        p3_view.destructively_redact_for_player(P3);
        let hidden_p1 = p3_view
            .river_crossing
            .as_ref()
            .unwrap()
            .players
            .iter()
            .find(|p| p.player_id == P1)
            .unwrap();
        assert!(hidden_p1.crossing_cards.is_empty());

        phase
            .submit_river_crossing_cards(P3, p3_cards.clone())
            .unwrap();
        phase
            .submit_river_return_cards(P1, p3_cards.clone())
            .unwrap();
        let mut p3_view = phase.clone();
        p3_view.destructively_redact_for_player(P3);
        let hidden_p1 = p3_view
            .river_crossing
            .as_ref()
            .unwrap()
            .players
            .iter()
            .find(|p| p.player_id == P1)
            .unwrap();
        assert!(hidden_p1.return_cards.is_empty());
    }

    #[test]
    fn no_trump_skips_river_crossing() {
        let mut phase = phase(
            vec![H_3, S_3, C_4, D_5, S_6],
            vec![H_4, S_7, C_8, D_9, S_10],
        );
        phase.trump = Trump::NoTrump {
            number: Some(Number::Two),
        };
        assert!(!phase.start_river_crossing(P1).unwrap());
        assert!(phase.river_crossing.is_none());
    }
}
