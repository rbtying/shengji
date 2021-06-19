use anyhow::{anyhow, bail, Error};
use serde::{Deserialize, Serialize};

use crate::hands::Hands;
use crate::player::Player;
use crate::types::{Card, PlayerID};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BidPolicy {
    JokerOrHigherSuit,
    JokerOrGreaterLength,
    GreaterLength,
}

impl Default for BidPolicy {
    fn default() -> Self {
        BidPolicy::JokerOrGreaterLength
    }
}
impl_slog_value!(BidPolicy);

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

impl_slog_value!(JokerBidPolicy);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BidReinforcementPolicy {
    /// A bid can be reinforced when it is the winning bid.
    ReinforceWhileWinning,
    /// A bid can be reinforced when it is the winning bid, or overturned with a greater bid.
    OverturnOrReinforceWhileWinning,
    /// A bid can be reinforced if it is equivalent to the winning bid after reinforcement.
    ReinforceWhileEquivalent,
}

impl Default for BidReinforcementPolicy {
    fn default() -> Self {
        BidReinforcementPolicy::ReinforceWhileWinning
    }
}
impl_slog_value!(BidReinforcementPolicy);

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

impl_slog_value!(BidTakebackPolicy);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Bid {
    pub(crate) id: PlayerID,
    pub(crate) card: Card,
    pub(crate) count: usize,
    #[serde(default)]
    pub(crate) epoch: usize,
}

impl Bid {
    #[allow(clippy::comparison_chain)]
    #[allow(clippy::too_many_arguments)]
    pub fn valid_bids(
        id: PlayerID,
        bids: &'_ [Bid],
        hands: &'_ Hands,
        players: &'_ [Player],
        landlord: Option<PlayerID>,
        epoch: usize,
        bid_policy: BidPolicy,
        bid_reinforcement_policy: BidReinforcementPolicy,
        joker_bid_policy: JokerBidPolicy,
        num_decks: usize,
    ) -> Result<Vec<Bid>, Error> {
        // Compute all valid bids.
        let most_recent_bid = bids.iter().rev().find(|b| b.id == id);
        let bid_player_id = landlord.unwrap_or(id);
        let bid_level = players
            .iter()
            .find(|p| p.id == bid_player_id)
            .map(|p| p.rank());

        let valid_bids = hands.counts(id).map(|counts| {
            // Construct all the valid bids from the player's hand
            let mut valid_bids = vec![];
            for (card, count) in counts {
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
                            match bid_policy {
                                BidPolicy::JokerOrHigherSuit | BidPolicy::JokerOrGreaterLength => {
                                    match (new_bid.card, existing_bid.card) {
                                        (Card::BigJoker, Card::BigJoker) => (),
                                        (Card::BigJoker, _) => valid_bids.push(new_bid),
                                        (Card::SmallJoker, Card::BigJoker)
                                        | (Card::SmallJoker, Card::SmallJoker) => (),
                                        (Card::SmallJoker, _) => valid_bids.push(new_bid),
                                        _ => {
                                            // The new bid count must have a size of at least 2 in
                                            // order to be compared by suit ranking
                                            if bid_policy == BidPolicy::JokerOrHigherSuit
                                                && new_bid.card.suit() > existing_bid.card.suit()
                                                && new_bid.count > 1
                                            {
                                                valid_bids.push(new_bid)
                                            }
                                        }
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
            valid_bids
        });

        match (most_recent_bid, bid_reinforcement_policy, valid_bids) {
            (Some(most_recent_bid), BidReinforcementPolicy::ReinforceWhileWinning, _)
            | (Some(most_recent_bid), BidReinforcementPolicy::ReinforceWhileEquivalent, _)
                if Some(most_recent_bid) == bids.last() =>
            {
                // If we're the current highest bidder, the only permissible bid is
                // one which is the same as the previous one, but has more cards
                let available = hands
                    .counts(id)
                    .and_then(|c| c.get(&most_recent_bid.card).cloned())
                    .unwrap_or(0);
                Ok((most_recent_bid.count + 1..=available)
                    .map(|count| Bid {
                        card: most_recent_bid.card,
                        count,
                        id,
                        epoch,
                    })
                    .collect())
            }
            (
                Some(most_recent_bid),
                BidReinforcementPolicy::ReinforceWhileEquivalent,
                Some(mut valid_bids),
            ) => {
                // If we can reinforce our bid to make a bid which is "equivalent" to the existing
                // bid, we should permit that as well.
                let available = hands
                    .counts(id)
                    .and_then(|c| c.get(&most_recent_bid.card).cloned())
                    .unwrap_or(0);

                if let Some(last_bid) = bids.last() {
                    if last_bid.count <= available {
                        let new_bid = Bid {
                            card: most_recent_bid.card,
                            count: last_bid.count,
                            id,
                            epoch,
                        };
                        if new_bid == *last_bid || !last_bid.card.is_joker() {
                            valid_bids.push(new_bid);
                        }
                    }
                }

                Ok(valid_bids)
            }
            (
                Some(_),
                BidReinforcementPolicy::OverturnOrReinforceWhileWinning,
                Some(valid_bids),
            )
            | (Some(_), BidReinforcementPolicy::ReinforceWhileWinning, Some(valid_bids))
            | (None, _, Some(valid_bids)) => Ok(valid_bids),
            (_, _, None) => Ok(vec![]),
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
        bid_reinforcement_policy: BidReinforcementPolicy,
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
            bid_reinforcement_policy,
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::hands::Hands;
    use crate::player::Player;
    use crate::types::{
        cards::{C_2, D_2, H_2, S_2},
        Card, PlayerID,
    };

    use super::{Bid, BidPolicy, BidReinforcementPolicy, JokerBidPolicy};

    macro_rules! b {
        ($p:expr, $card:expr, $count:expr) => {
            Bid {
                id: $p,
                card: $card,
                count: $count,
                epoch: 0,
            }
        };
    }

    #[test]
    fn test_valid_bids() {
        let p = PlayerID(0);
        let mut h = Hands::new(vec![p]);
        h.add(
            p,
            vec![
                C_2,
                C_2,
                C_2,
                S_2,
                S_2,
                Card::SmallJoker,
                Card::SmallJoker,
                Card::BigJoker,
                Card::BigJoker,
            ],
        )
        .unwrap();
        let players = vec![Player::new(p, "p0".into())];

        let test_cases = vec![
            (
                vec![],
                BidReinforcementPolicy::ReinforceWhileWinning,
                vec![
                    b!(p, S_2, 1),
                    b!(p, S_2, 2),
                    b!(p, C_2, 1),
                    b!(p, C_2, 2),
                    b!(p, C_2, 3),
                    b!(p, Card::BigJoker, 2),
                    b!(p, Card::SmallJoker, 2),
                ],
            ),
            // We should only be able to reinforce our bid if we are winning for ReinforceWhileWinning
            // and ReinforceWhileEquivalent.
            (
                vec![b!(p, C_2, 1)],
                BidReinforcementPolicy::ReinforceWhileWinning,
                vec![b!(p, C_2, 2), b!(p, C_2, 3)],
            ),
            (
                vec![b!(p, C_2, 1)],
                BidReinforcementPolicy::ReinforceWhileEquivalent,
                vec![b!(p, C_2, 2), b!(p, C_2, 3)],
            ),
            // If we have OverturnOrReinforceWhileWinning, we can bid anything which is higher.
            (
                vec![b!(p, C_2, 1)],
                BidReinforcementPolicy::OverturnOrReinforceWhileWinning,
                vec![
                    b!(p, S_2, 2),
                    b!(p, C_2, 2),
                    b!(p, C_2, 3),
                    b!(p, Card::BigJoker, 2),
                    b!(p, Card::SmallJoker, 2),
                ],
            ),
            // If somebody else has defeated our bid and we can only reinforce while winning, we have
            // to bid something higher.
            (
                vec![b!(p, C_2, 1), b!(PlayerID(1), S_2, 2)],
                BidReinforcementPolicy::ReinforceWhileWinning,
                vec![
                    b!(p, C_2, 3),
                    b!(p, Card::BigJoker, 2),
                    b!(p, Card::SmallJoker, 2),
                ],
            ),
            (
                vec![b!(p, C_2, 1), b!(PlayerID(1), S_2, 2)],
                BidReinforcementPolicy::OverturnOrReinforceWhileWinning,
                vec![
                    b!(p, C_2, 3),
                    b!(p, Card::BigJoker, 2),
                    b!(p, Card::SmallJoker, 2),
                ],
            ),
            // If we can reinforce while equivalent, we can bid the same amount.
            (
                vec![b!(p, C_2, 1), b!(PlayerID(1), S_2, 2)],
                BidReinforcementPolicy::ReinforceWhileEquivalent,
                vec![
                    b!(p, C_2, 2),
                    b!(p, C_2, 3),
                    b!(p, Card::BigJoker, 2),
                    b!(p, Card::SmallJoker, 2),
                ],
            ),
            // We still need to do better if the conflicting behavior is for jokers.
            (
                vec![b!(p, C_2, 1), b!(PlayerID(1), Card::SmallJoker, 2)],
                BidReinforcementPolicy::ReinforceWhileEquivalent,
                vec![b!(p, C_2, 3), b!(p, Card::BigJoker, 2)],
            ),
        ];

        for (bids, rpol, results) in test_cases {
            assert_eq!(
                Bid::valid_bids(
                    p,
                    &bids,
                    &h,
                    &players,
                    None,
                    0,
                    BidPolicy::JokerOrGreaterLength,
                    rpol,
                    JokerBidPolicy::BothTwoOrMore,
                    3,
                )
                .unwrap()
                .into_iter()
                .collect::<HashSet<_>>(),
                results.into_iter().collect::<HashSet<_>>()
            );
        }
    }

    #[test]
    fn test_valid_bids_joker_or_higher_suit() {
        let p = PlayerID(0);
        let mut h = Hands::new(vec![p]);
        h.add(
            p,
            vec![
                C_2,
                C_2,
                C_2,
                S_2,
                S_2,
                Card::SmallJoker,
                Card::SmallJoker,
                Card::BigJoker,
                Card::BigJoker,
            ],
        )
        .unwrap();
        let players = vec![Player::new(p, "p0".into())];

        let test_cases_higher_suit = vec![
            (
                vec![b!(p, C_2, 1), b!(PlayerID(1), S_2, 2)],
                BidReinforcementPolicy::ReinforceWhileWinning,
                vec![
                    b!(p, C_2, 3),
                    b!(p, Card::BigJoker, 2),
                    b!(p, Card::SmallJoker, 2),
                ],
            ),
            (
                vec![b!(p, C_2, 1), b!(PlayerID(1), H_2, 2)],
                BidReinforcementPolicy::ReinforceWhileWinning,
                vec![
                    b!(p, S_2, 2),
                    b!(p, C_2, 3),
                    b!(p, Card::BigJoker, 2),
                    b!(p, Card::SmallJoker, 2),
                ],
            ),
            (
                vec![b!(p, C_2, 1), b!(PlayerID(1), D_2, 2)],
                BidReinforcementPolicy::ReinforceWhileWinning,
                vec![
                    b!(p, S_2, 2),
                    b!(p, C_2, 2),
                    b!(p, C_2, 3),
                    b!(p, Card::BigJoker, 2),
                    b!(p, Card::SmallJoker, 2),
                ],
            ),
            (
                vec![b!(p, C_2, 1), b!(PlayerID(1), D_2, 3)],
                BidReinforcementPolicy::ReinforceWhileWinning,
                vec![b!(p, C_2, 3)],
            ),
        ];

        for (bids, rpol, results) in test_cases_higher_suit {
            assert_eq!(
                Bid::valid_bids(
                    p,
                    &bids,
                    &h,
                    &players,
                    None,
                    0,
                    BidPolicy::JokerOrHigherSuit,
                    rpol,
                    JokerBidPolicy::BothTwoOrMore,
                    3,
                )
                .unwrap()
                .into_iter()
                .collect::<HashSet<_>>(),
                results.into_iter().collect::<HashSet<_>>()
            );
        }
    }
}
