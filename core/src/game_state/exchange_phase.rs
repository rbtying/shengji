use std::collections::HashSet;

use anyhow::{anyhow, bail, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::bidding::Bid;
use crate::deck::Deck;
use crate::hands::Hands;
use crate::message::MessageVariant;
use crate::settings::{
    Friend, FriendSelection, FriendSelectionPolicy, GameMode, KittyTheftPolicy, PropagatedState,
};
use crate::types::{Card, Number, PlayerID, Rank, Trump};

use crate::game_state::{initialize_phase::InitializePhase, play_phase::PlayPhase};

macro_rules! bail_unwrap {
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => return Err(anyhow!("option was none")),
        }
    };
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
            finalized: false,
            epoch: 1,
        }
    }

    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

    pub fn move_card_to_kitty(&mut self, id: PlayerID, card: Card) -> Result<(), Error> {
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
                number: self
                    .trump
                    .number()
                    .expect("Shouldn't have trump number if there are bids"),
            },
        };
        self.finalized = false;
        self.epoch += 1;
        self.exchanger = winning_bid.id;

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

    pub fn propagated(&self) -> &PropagatedState {
        &self.propagated
    }

    pub fn propagated_mut(&mut self) -> &mut PropagatedState {
        &mut self.propagated
    }

    pub fn next_player(&self) -> Result<PlayerID, Error> {
        if self.propagated.kitty_theft_policy == KittyTheftPolicy::AllowKittyTheft
            && self.autobid.is_none()
            && !self.finalized
        {
            Ok(self.exchanger)
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
        )
    }

    pub fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase::from_propagated(propagated), msgs))
    }

    pub fn destructively_redact_for_player(&mut self, player: PlayerID) {
        self.hands.destructively_redact_except_for_player(player);
        if player != self.exchanger || self.finalized {
            for card in &mut self.kitty {
                *card = Card::Unknown;
            }
        }
        if player != self.landlord {
            if let GameMode::FindingFriends {
                ref mut friends, ..
            } = self.game_mode
            {
                friends.clear();
            }
        }
    }
}
