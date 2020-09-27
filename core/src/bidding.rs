use anyhow::{anyhow, bail, Error};
use serde::{Deserialize, Serialize};

use crate::hands::Hands;
use crate::player::Player;
use crate::types::{Card, PlayerID};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BidPolicy {
    JokerOrGreaterLength,
    GreaterLength,
}

impl Default for BidPolicy {
    fn default() -> Self {
        BidPolicy::JokerOrGreaterLength
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum JokerBidPolicy {
    BothTwoOrMore,
    BothNumDecks,
    LJNumDecksHJNumDecksLessOne,
}

impl Default for JokerBidPolicy {
    fn default() -> Self {
        JokerBidPolicy::BothTwoOrMore
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BidTakebackPolicy {
    AllowBidTakeback,
    NoBidTakeback,
}

impl Default for BidTakebackPolicy {
    fn default() -> Self {
        BidTakebackPolicy::AllowBidTakeback
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bid {
    pub(crate) id: PlayerID,
    pub(crate) card: Card,
    pub(crate) count: usize,
    #[serde(default)]
    pub(crate) epoch: usize,
}

impl Bid {
    #[allow(clippy::comparison_chain)]
    pub fn valid_bids(
        id: PlayerID,
        bids: &'_ [Bid],
        hands: &'_ Hands,
        players: &'_ [Player],
        landlord: Option<PlayerID>,
        epoch: usize,
        bid_policy: BidPolicy,
        joker_bid_policy: JokerBidPolicy,
        num_decks: usize,
    ) -> Result<Vec<Bid>, Error> {
        // Compute all valid bids.
        if bids.last().map(|b| b.id) == Some(id) {
            // If we're the current highest bidder, the only permissible bid is
            // one which is the same as the previous one, but has more cards
            let last_bid = bids.last().ok_or_else(|| anyhow!("no highest bid?"))?;
            let available = hands
                .counts(id)
                .and_then(|c| c.get(&last_bid.card).cloned())
                .unwrap_or(0);
            Ok((last_bid.count + 1..=available)
                .map(|count| Bid {
                    card: last_bid.card,
                    count,
                    id,
                    epoch,
                })
                .collect())
        } else if let Some(counts) = hands.counts(id) {
            // Construct all the valid bids from the player's hand
            let mut valid_bids = vec![];
            for (card, count) in counts {
                let bid_player_id = landlord.unwrap_or(id);
                let bid_level = players
                    .iter()
                    .find(|p| p.id == bid_player_id)
                    .map(|p| p.rank());
                if !card.is_joker() && card.number() != bid_level {
                    continue;
                }
                for inner_count in 1..=*count {
                    if card.is_joker() {
                        match (card, joker_bid_policy) {
                            (_, JokerBidPolicy::BothTwoOrMore) if inner_count <= 1 => continue,
                            (Card::SmallJoker, JokerBidPolicy::LJNumDecksHJNumDecksLessOne)
                            | (Card::SmallJoker, JokerBidPolicy::BothNumDecks)
                                if inner_count < num_decks =>
                            {
                                continue
                            }
                            (Card::BigJoker, JokerBidPolicy::LJNumDecksHJNumDecksLessOne)
                                if inner_count < num_decks - 1 =>
                            {
                                continue
                            }
                            (Card::BigJoker, JokerBidPolicy::BothNumDecks)
                                if inner_count < num_decks =>
                            {
                                continue
                            }
                            (_, _) => (),
                        }
                    }
                    let new_bid = Bid {
                        id,
                        card: *card,
                        count: inner_count,
                        epoch,
                    };
                    if let Some(existing_bid) = bids.last() {
                        if new_bid.count > existing_bid.count {
                            valid_bids.push(new_bid);
                        } else if new_bid.count == existing_bid.count {
                            match (new_bid.card, existing_bid.card) {
                                (Card::BigJoker, Card::BigJoker) => (),
                                (Card::BigJoker, _) => {
                                    if bid_policy == BidPolicy::JokerOrGreaterLength {
                                        valid_bids.push(new_bid)
                                    }
                                }
                                (Card::SmallJoker, Card::BigJoker)
                                | (Card::SmallJoker, Card::SmallJoker) => (),
                                (Card::SmallJoker, _) => {
                                    if bid_policy == BidPolicy::JokerOrGreaterLength {
                                        valid_bids.push(new_bid)
                                    }
                                }
                                _ => (),
                            }
                        }
                    } else {
                        valid_bids.push(new_bid);
                    }
                }
            }

            Ok(valid_bids)
        } else {
            Ok(vec![])
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bid(
        id: PlayerID,
        card: Card,
        count: usize,
        bids: &'_ mut Vec<Bid>,
        autobid: Option<Bid>,
        hands: &'_ Hands,
        players: &'_ [Player],
        landlord: Option<PlayerID>,
        bid_policy: BidPolicy,
        joker_bid_policy: JokerBidPolicy,
        num_decks: usize,
        epoch: usize,
    ) -> bool {
        if autobid.is_some() {
            return false;
        }

        let new_bid = Bid {
            id,
            card,
            count,
            epoch,
        };
        if Self::valid_bids(
            id,
            bids,
            hands,
            players,
            landlord,
            epoch,
            bid_policy,
            joker_bid_policy,
            num_decks,
        )
        .map(|b| b.contains(&new_bid))
        .unwrap_or(false)
        {
            bids.push(new_bid);
            true
        } else {
            false
        }
    }

    pub fn take_back_bid(
        id: PlayerID,
        bid_takeback_policy: BidTakebackPolicy,
        bids: &'_ mut Vec<Bid>,
        epoch: usize,
    ) -> Result<(), Error> {
        if bid_takeback_policy == BidTakebackPolicy::NoBidTakeback {
            bail!("Taking back bids is not allowed!")
        }
        if bids.last().map(|b| (b.id, b.epoch)) == Some((id, epoch)) {
            bids.pop();
            Ok(())
        } else {
            bail!("Can't do that right now")
        }
    }

    /// Returns the player IDs for the first player to bid, and for the player who won the bid.
    pub fn first_and_winner(bids: &'_ [Bid], autobid: Option<Bid>) -> Result<(Bid, Bid), Error> {
        if bids.is_empty() && autobid.is_none() {
            bail!("nobody has bid yet")
        }

        let winning_bid = autobid
            .or_else(|| bids.last().copied())
            .ok_or_else(|| anyhow!("No winning bid found!"))?;
        let first_bid = bids
            .first()
            .copied()
            .or(autobid)
            .ok_or_else(|| anyhow!("No bid found!"))?;
        Ok((first_bid, winning_bid))
    }
}
