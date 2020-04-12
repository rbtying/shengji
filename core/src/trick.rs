use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::game_state::MessageVariant;
use crate::hands::{HandError, Hands};
use crate::types::{Card, EffectiveSuit, PlayerID, Trump};

#[derive(Error, Clone, Debug, Serialize, Deserialize)]
pub enum TrickError {
    #[error("error in hand {}", source)]
    HandError {
        #[from]
        source: HandError,
    },
    #[error("wrong number of cards provided")]
    WrongNumberOfCards,
    #[error("the cards have the wrong number of suits")]
    WrongNumberOfSuits,
    #[error("player is playing out of order")]
    OutOfOrder,
    #[error("this play is illegal")]
    IllegalPlay,
    #[error("this play doesn't match the format")]
    NonMatchingPlay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrickUnit {
    Tractor { count: usize, members: Vec<Card> },
    Repeated { count: usize, card: Card },
}

impl TrickUnit {
    pub fn is_tractor(&self) -> bool {
        match self {
            TrickUnit::Tractor { .. } => true,
            TrickUnit::Repeated { .. } => false,
        }
    }

    pub fn is_repeated(&self) -> bool {
        match self {
            TrickUnit::Tractor { .. } => false,
            TrickUnit::Repeated { .. } => true,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            TrickUnit::Tractor { count, members } => members.len() * *count,
            TrickUnit::Repeated { count, .. } => *count,
        }
    }

    pub fn first_card(&self) -> Card {
        match self {
            TrickUnit::Tractor { members, .. } => members[0],
            TrickUnit::Repeated { card, .. } => *card,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrickFormat {
    suit: EffectiveSuit,
    trump: Trump,
    units: Vec<TrickUnit>,
}

impl TrickFormat {
    pub fn is_legal_play(&self, hand: &HashMap<Card, usize>, proposed: &'_ [Card]) -> bool {
        let required = self.units.iter().map(|c| c.size()).sum::<usize>();
        if proposed.len() != required {
            return false;
        }

        let num_proposed_correct_suit = proposed
            .iter()
            .filter(|c| self.trump.effective_suit(**c) == self.suit)
            .count();

        if num_proposed_correct_suit < required {
            let num_correct_suit = hand
                .iter()
                .flat_map(|(c, ct)| {
                    if self.trump.effective_suit(*c) == self.suit {
                        Some(*ct)
                    } else {
                        None
                    }
                })
                .sum::<usize>();
            // If this is all of the correct suit that is available, it's fine
            // Otherwise, this is an invalid play.
            num_correct_suit == num_proposed_correct_suit
        } else {
            // If it's a match, we're good!
            let counts = Card::count(proposed.iter().cloned());
            if check_format_matches(self.trump, &self.units, counts.clone()) {
                return true;
            }
            let available: HashMap<Card, usize> = hand
                .iter()
                .flat_map(|(c, ct)| {
                    if self.trump.effective_suit(*c) == self.suit {
                        Some((*c, *ct))
                    } else {
                        None
                    }
                })
                .collect();
            if check_format_matches(self.trump, &self.units, available.clone()) {
                return false;
            }

            // Check if we meet requirements if we replace all tractors with repeated
            let mut requirements = self
                .units
                .iter()
                .flat_map(|unit| match unit {
                    TrickUnit::Tractor { members, count } => members
                        .iter()
                        .map(|card| TrickUnit::Repeated {
                            count: *count,
                            card: *card,
                        })
                        .collect(),
                    TrickUnit::Repeated { card, count } if *count > 1 => {
                        vec![TrickUnit::Repeated {
                            count: *count,
                            card: *card,
                        }]
                    }
                    _ => vec![],
                })
                .collect::<Vec<_>>();

            loop {
                requirements.sort_by(|a, b| {
                    a.size()
                        .cmp(&b.size())
                        .then(self.trump.compare(a.first_card(), b.first_card()))
                });
                if !check_format_matches(self.trump, &requirements, counts.clone()) {
                    if check_format_matches(self.trump, &requirements, available.clone()) {
                        break false;
                    } else {
                        // reduce requirements more
                        match requirements.pop() {
                            Some(TrickUnit::Repeated { card, count }) if count > 2 => {
                                requirements.push(TrickUnit::Repeated {
                                    card,
                                    count: count - 1,
                                });
                            }
                            _ => (),
                        }
                    }
                } else {
                    break true;
                }
            }
        }
    }

    pub fn matches(&self, cards: &'_ [Card]) -> Result<Vec<TrickUnit>, TrickError> {
        let suit = self.trump.effective_suit(cards[0]);
        for card in cards {
            if self.trump.effective_suit(*card) != suit {
                return Err(TrickError::NonMatchingPlay);
            }
        }

        if suit != self.suit && suit != EffectiveSuit::Trump {
            return Err(TrickError::NonMatchingPlay);
        }

        if cards.len() != self.units.iter().map(|u| u.size()).sum::<usize>() {
            return Err(TrickError::NonMatchingPlay);
        }

        let counts = Card::count(cards.iter().cloned());
        check_format_matches_mapping(self.trump, &self.units, counts)
            .ok_or(TrickError::NonMatchingPlay)
    }

    pub fn from_cards(trump: Trump, cards: &'_ [Card]) -> Result<TrickFormat, TrickError> {
        if cards.is_empty() {
            return Err(TrickError::WrongNumberOfSuits);
        }
        let suit = trump.effective_suit(cards[0]);
        let mut all_cards_match = true;
        for card in cards {
            if trump.effective_suit(*card) != suit {
                return Err(TrickError::WrongNumberOfSuits);
            }
            if *card != cards[0] {
                all_cards_match = false;
            }
        }
        // Handle simple cases
        if all_cards_match {
            Ok(TrickFormat {
                suit,
                trump,
                units: vec![TrickUnit::Repeated {
                    count: cards.len(),
                    card: cards[0],
                }],
            })
        } else {
            // The generalized trick format is actually ambiguous here.
            // Let's use something *really* inefficient for now.

            // 1. Find all of the tractors
            let mut counts = Card::count(cards.iter().cloned());
            let mut units = vec![];
            loop {
                let mut tractors = find_tractors(trump, &counts);

                // If the tractor is shorter than one of its segments alone, don't
                // include it.
                tractors.retain(
                    |FoundTractor {
                         ref members, size, ..
                     }| {
                        *size >= members.iter().map(|cc| counts[&cc]).max().unwrap()
                    },
                );

                match tractors.pop() {
                    Some(FoundTractor { members, count, .. }) => {
                        // reduce the counts appropriately
                        for card in &members {
                            *counts.get_mut(card).unwrap() -= count;
                        }
                        units.push(TrickUnit::Tractor { count, members });
                    }
                    None => break,
                }
            }

            // Mark everything remaining as `repeated`
            for (card, count) in counts {
                if count > 0 {
                    units.push(TrickUnit::Repeated { count, card });
                }
            }
            units.sort_by(|a, b| {
                a.size()
                    .cmp(&b.size())
                    .then(trump.compare(a.first_card(), b.first_card()))
            });

            Ok(TrickFormat { suit, units, trump })
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlayedCards {
    pub id: PlayerID,
    pub cards: Vec<Card>,
    pub bad_throw_cards: Vec<Card>,
    pub better_player: Option<PlayerID>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trick {
    player_queue: VecDeque<PlayerID>,
    played_cards: Vec<PlayedCards>,
    current_winner: Option<PlayerID>,
    trick_format: Option<TrickFormat>,
    trump: Trump,
}

impl Trick {
    pub fn new(trump: Trump, players: impl IntoIterator<Item = PlayerID>) -> Self {
        Trick {
            player_queue: players.into_iter().collect(),
            played_cards: vec![],
            current_winner: None,
            trick_format: None,
            trump,
        }
    }

    pub fn played_cards(&self) -> &'_ [PlayedCards] {
        &self.played_cards
    }

    pub fn next_player(&self) -> Option<PlayerID> {
        self.player_queue.front().cloned()
    }

    pub fn player_queue(&self) -> impl Iterator<Item = PlayerID> + '_ {
        self.player_queue.iter().copied()
    }

    /**
     * Determines whether the player can play the cards.
     *
     * Note: this does not account throw validity, nor is it intended to catch all illegal plays.
     */
    pub fn can_play_cards<'a, 'b>(
        &self,
        id: PlayerID,
        hands: &'a Hands,
        cards: &'b [Card],
    ) -> Result<(), TrickError> {
        hands.contains(id, cards.iter().cloned())?;
        if self.player_queue.front().cloned() != Some(id) {
            return Err(TrickError::OutOfOrder);
        }
        match self.trick_format.as_ref() {
            Some(tf) => {
                if tf.is_legal_play(hands.get(id)?, cards) {
                    Ok(())
                } else {
                    Err(TrickError::IllegalPlay)
                }
            }
            None => {
                let num_suits = cards
                    .iter()
                    .map(|c| self.trump.effective_suit(*c))
                    .collect::<HashSet<EffectiveSuit>>()
                    .len();
                if num_suits == 1 {
                    Ok(())
                } else {
                    Err(TrickError::WrongNumberOfSuits)
                }
            }
        }
    }

    /**
     * Actually plays the cards, if possible. On error, does not modify any state.
     *
     * Note: this does not account throw validity, nor is it intended to catch all illegal plays.
     */
    pub fn play_cards<'a, 'b>(
        &mut self,
        id: PlayerID,
        hands: &'a mut Hands,
        cards: &'b [Card],
    ) -> Result<Vec<MessageVariant>, TrickError> {
        self.can_play_cards(id, hands, cards)?;
        let mut msgs = vec![];
        let mut cards = cards.to_vec();
        cards.sort_by(|a, b| self.trump.compare(*a, *b));

        let (cards, bad_throw_cards, better_player) = if self.trick_format.is_none() {
            let mut tf = TrickFormat::from_cards(self.trump, &cards)?;
            let mut invalid = None;
            if tf.units.len() > 1 {
                // This is a throw, let's see if any of the units can be strictly defeated by any
                // other player.
                'search: for player in self.player_queue.iter().skip(1) {
                    for unit in &tf.units {
                        match unit {
                            TrickUnit::Repeated { count, card } => {
                                for (c, ct) in hands.get(*player)? {
                                    if self.trump.effective_suit(*c) == tf.suit
                                        && ct >= count
                                        && self.trump.compare(*c, *card) == Ordering::Greater
                                    {
                                        invalid = Some((player, unit.clone()));
                                        break 'search;
                                    }
                                }
                            }
                            TrickUnit::Tractor { count, members } => {
                                for FoundTractor {
                                    members: found_members,
                                    count: found_count,
                                    ..
                                } in find_tractors(self.trump, hands.get(*player)?)
                                {
                                    if self.trump.effective_suit(members[0]) == tf.suit
                                        && found_count >= *count
                                        && found_members.len() >= members.len()
                                    {
                                        invalid = Some((player, unit.clone()));
                                        break 'search;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            cards.sort_by(|a, b| self.trump.compare(*a, *b));
            let (cards, bad_throw_cards, better_player) =
                if let Some((better_player, forced_unit)) = invalid {
                    let forced_cards: Vec<Card> = match forced_unit {
                        TrickUnit::Repeated { card, count } => (0..count).map(|_| card).collect(),
                        TrickUnit::Tractor { ref members, count } => members
                            .iter()
                            .flat_map(|card| (0..count).map(move |_| *card))
                            .collect(),
                    };

                    tf.units = vec![forced_unit];

                    msgs.push(MessageVariant::ThrowFailed {
                        original_cards: cards.clone(),
                        better_player: *better_player,
                    });

                    for card in &forced_cards {
                        let idx = cards.iter().position(|c| *c == *card).unwrap();
                        cards.remove(idx);
                    }

                    (forced_cards, cards, Some(*better_player))
                } else {
                    (cards, vec![], None)
                };

            self.trick_format = Some(tf);

            msgs.push(MessageVariant::PlayedCards {
                cards: cards.clone(),
            });

            (cards, bad_throw_cards, better_player)
        } else {
            msgs.push(MessageVariant::PlayedCards {
                cards: cards.clone(),
            });
            (cards, vec![], None)
        };

        hands.remove(id, cards.iter().cloned())?;

        self.player_queue.pop_front();
        self.played_cards.push(PlayedCards {
            id,
            cards,
            bad_throw_cards,
            better_player,
        });

        self.current_winner =
            Self::winner(self.trick_format.as_ref(), &self.played_cards, self.trump);
        Ok(msgs)
    }

    /**
     * Takes back cards just played, e.g. in case of dispute.
     */
    pub fn take_back(&mut self, id: PlayerID, hands: &'_ mut Hands) -> Result<(), TrickError> {
        if self.played_cards.last().map(|p| p.id) == Some(id) {
            let played = self.played_cards.pop().unwrap();
            hands.add(id, played.cards).unwrap();
            self.player_queue.push_front(id);
            if self.played_cards.is_empty() {
                self.trick_format = None;
            }
            self.current_winner =
                Self::winner(self.trick_format.as_ref(), &self.played_cards, self.trump);
            Ok(())
        } else {
            Err(TrickError::OutOfOrder)
        }
    }

    /**
     * Completes the trick and determines the winner. Returns the point cards that the winner won.
     */
    pub fn complete(&self) -> Result<TrickEnded, TrickError> {
        if !self.player_queue.is_empty() || self.played_cards.is_empty() {
            return Err(TrickError::OutOfOrder);
        }
        if let Some(tf) = self.trick_format.as_ref() {
            let all_card_points = self
                .played_cards
                .iter()
                .flat_map(|pc| pc.cards.iter().filter(|c| c.points().is_some()).copied())
                .collect::<Vec<Card>>();

            Ok(TrickEnded {
                winner: self.current_winner.ok_or(TrickError::OutOfOrder)?,
                points: all_card_points,
                largest_trick_unit_size: tf.units.iter().map(|u| u.size()).max().unwrap_or(0),
                failed_throw_size: self
                    .played_cards
                    .get(0)
                    .ok_or(TrickError::OutOfOrder)?
                    .bad_throw_cards
                    .len(),
            })
        } else {
            Err(TrickError::OutOfOrder)
        }
    }

    fn winner(
        trick_format: Option<&'_ TrickFormat>,
        played_cards: &'_ [PlayedCards],
        trump: Trump,
    ) -> Option<PlayerID> {
        match trick_format {
            Some(tf) => {
                let mut winner = (0, tf.units.clone());

                for (idx, pc) in played_cards.iter().enumerate().skip(1) {
                    if let Ok(m) = tf.matches(&pc.cards) {
                        let all_greater = m.iter().zip(winner.1.iter()).all(|(n, w)| {
                            trump.compare(n.first_card(), w.first_card()) == Ordering::Greater
                        });
                        if all_greater {
                            winner = (idx, m);
                        }
                    }
                }
                Some(played_cards[winner.0].id)
            }
            None => None,
        }
    }
}

pub struct TrickEnded {
    pub winner: PlayerID,
    pub points: Vec<Card>,
    pub largest_trick_unit_size: usize,
    pub failed_throw_size: usize,
}

#[derive(Debug)]
pub struct FoundTractor {
    members: Vec<Card>,
    count: usize,
    size: usize,
}

pub fn find_tractors(trump: Trump, counts: &HashMap<Card, usize>) -> Vec<FoundTractor> {
    let mut tractors = vec![];
    for (card, count) in counts {
        if *count <= 1 {
            continue;
        }

        // DFS for possible tractors
        let mut stk = vec![(*card, vec![*card])];
        while let Some((c, p)) = stk.pop() {
            for cc in trump.successor(c) {
                if counts.get(&cc).cloned().unwrap_or(0) > 1 {
                    let mut pp = p.clone();
                    pp.push(cc);
                    stk.push((cc, pp));
                }
            }
            if p.len() >= 2 {
                // This is a tractor!
                let min_count = p.iter().map(|cc| counts[&cc]).min().unwrap();
                let size = p.len() * min_count;
                tractors.push(FoundTractor {
                    members: p,
                    count: min_count,
                    size,
                });
            }
        }
    }

    // Sort the tractors by the number of available cards
    tractors.sort_by_key(|FoundTractor { size, .. }| *size);
    tractors
}

pub fn check_format_matches(
    trump: Trump,
    units: &'_ [TrickUnit],
    mut counts: HashMap<Card, usize>,
) -> bool {
    check_format_matches_inner(trump, units, &mut counts, &mut vec![])
}

pub fn check_format_matches_mapping(
    trump: Trump,
    units: &'_ [TrickUnit],
    mut counts: HashMap<Card, usize>,
) -> Option<Vec<TrickUnit>> {
    // Start out with dummy values.
    let mut matches = units
        .iter()
        .map(|_| TrickUnit::Repeated {
            count: 0,
            card: Card::BigJoker,
        })
        .collect();
    let matched = check_format_matches_inner(trump, units, &mut counts, &mut matches);
    if matched {
        matches.sort_by(|a, b| {
            a.size()
                .cmp(&b.size())
                .then(trump.compare(a.first_card(), b.first_card()))
        });
        Some(matches)
    } else {
        None
    }
}

fn check_format_matches_inner(
    trump: Trump,
    units: &'_ [TrickUnit],
    counts: &'_ mut HashMap<Card, usize>,
    allocations: &'_ mut Vec<TrickUnit>,
) -> bool {
    if let Some(unit) = units.last() {
        match unit {
            TrickUnit::Tractor { members, count } => {
                let mut available_tractors = find_tractors(trump, &counts);
                available_tractors.retain(
                    |FoundTractor {
                         members: t,
                         count: ct,
                         ..
                     }| t.len() == members.len() && *ct == *count,
                );
                for FoundTractor { members, count, .. } in available_tractors {
                    for card in &members {
                        *counts.get_mut(card).unwrap() -= count;
                    }
                    if check_format_matches_inner(
                        trump,
                        &units[..units.len() - 1],
                        counts,
                        allocations,
                    ) {
                        if !allocations.is_empty() {
                            allocations[units.len() - 1] = TrickUnit::Tractor { members, count };
                        }
                        return true;
                    } else {
                        for card in &members {
                            *counts.get_mut(card).unwrap() += count;
                        }
                    }
                }
                false
            }
            TrickUnit::Repeated { count, .. } => {
                let viable_repeated = counts
                    .iter()
                    .filter(|(_, ct)| **ct >= *count)
                    .map(|(card, _)| *card)
                    .collect::<Vec<_>>();
                for card in viable_repeated {
                    *counts.get_mut(&card).unwrap() -= count;

                    if check_format_matches_inner(
                        trump,
                        &units[..units.len() - 1],
                        counts,
                        allocations,
                    ) {
                        if !allocations.is_empty() {
                            allocations[units.len() - 1] = TrickUnit::Repeated {
                                count: *count,
                                card,
                            };
                        }
                        return true;
                    } else {
                        *counts.get_mut(&card).unwrap() += count;
                    }
                }
                false
            }
        }
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Trick, TrickEnded, TrickFormat, TrickUnit};

    use crate::hands::Hands;
    use crate::types::{
        cards::{H_2, H_3, H_7, H_8, S_2, S_3, S_4, S_5, S_6, S_7, S_8},
        Card, EffectiveSuit, Number, PlayerID, Suit, Trump,
    };

    const TRUMP: Trump = Trump::Standard {
        number: Number::Four,
        suit: Suit::Spades,
    };
    const P1: PlayerID = PlayerID(1);
    const P2: PlayerID = PlayerID(2);
    const P3: PlayerID = PlayerID(3);
    const P4: PlayerID = PlayerID(4);

    #[test]
    fn test_play_singles_trick() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4], Number::Four);
        hands.add(P1, vec![S_2, S_3, S_5]).unwrap();
        hands.add(P2, vec![S_2, S_3, S_5]).unwrap();
        hands.add(P3, vec![S_2, S_3, S_5]).unwrap();
        hands.add(P4, vec![S_2, S_3, S_5]).unwrap();
        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);

        trick.play_cards(P1, &mut hands, &[S_2]).unwrap();
        trick.play_cards(P2, &mut hands, &[S_5]).unwrap();
        trick.play_cards(P3, &mut hands, &[S_3]).unwrap();
        trick.play_cards(P4, &mut hands, &[S_5]).unwrap();
        let TrickEnded {
            winner: winner_id,
            points,
            largest_trick_unit_size,
            ..
        } = trick.complete().unwrap();
        assert_eq!(winner_id, P2);
        assert_eq!(largest_trick_unit_size, 1);
        assert_eq!(points, vec![S_5, S_5]);
    }

    #[test]
    fn test_play_trump_trick() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4], Number::Four);
        hands.add(P1, vec![S_2, S_3, S_5]).unwrap();
        hands.add(P2, vec![H_2, H_3, S_4]).unwrap();
        hands.add(P3, vec![S_2, S_3, S_5]).unwrap();
        hands.add(P4, vec![S_2, S_3, S_5]).unwrap();
        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);

        trick.play_cards(P1, &mut hands, &[S_2]).unwrap();
        trick.play_cards(P2, &mut hands, &[S_4]).unwrap();
        trick.play_cards(P3, &mut hands, &[S_3]).unwrap();
        trick.play_cards(P4, &mut hands, &[S_5]).unwrap();
        let TrickEnded {
            winner: winner_id,
            points,
            largest_trick_unit_size,
            ..
        } = trick.complete().unwrap();
        assert_eq!(winner_id, P2);
        assert_eq!(largest_trick_unit_size, 1);
        assert_eq!(points, vec![S_5]);
    }

    #[test]
    fn test_play_pairs_trick() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4], Number::Four);
        hands.add(P1, vec![S_2, S_2, S_5]).unwrap();
        hands.add(P2, vec![H_2, S_3, S_4]).unwrap();
        hands.add(P3, vec![S_5, S_5, S_5]).unwrap();
        hands.add(P4, vec![S_3, S_4, S_5]).unwrap();
        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);

        trick.play_cards(P1, &mut hands, &[S_2, S_2]).unwrap();
        trick.play_cards(P2, &mut hands, &[S_3, S_4]).unwrap();
        trick.play_cards(P3, &mut hands, &[S_5, S_5]).unwrap();
        trick.play_cards(P4, &mut hands, &[S_3, S_5]).unwrap();
        let TrickEnded {
            winner: winner_id,
            points,
            largest_trick_unit_size,
            ..
        } = trick.complete().unwrap();
        assert_eq!(winner_id, P3);
        assert_eq!(largest_trick_unit_size, 2);
        assert_eq!(points, vec![S_5, S_5, S_5]);
    }

    #[test]
    fn test_play_tractor_trick() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4], Number::Four);
        hands.add(P1, vec![S_2, S_2, S_3, S_3, S_4]).unwrap();
        hands.add(P2, vec![S_6, S_6, S_7, S_7, S_4]).unwrap();
        hands.add(P3, vec![S_2, S_5, S_5, S_5, S_4]).unwrap();
        hands.add(P4, vec![S_6, S_6, S_6, S_6, S_4]).unwrap();
        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);

        trick
            .play_cards(P1, &mut hands, &[S_2, S_2, S_3, S_3])
            .unwrap();
        trick
            .play_cards(P2, &mut hands, &[S_6, S_6, S_7, S_7])
            .unwrap();
        trick
            .play_cards(P3, &mut hands, &[S_2, S_5, S_5, S_5])
            .unwrap();
        trick
            .play_cards(P4, &mut hands, &[S_6, S_6, S_6, S_6])
            .unwrap();
        let TrickEnded {
            winner: winner_id,
            points,
            largest_trick_unit_size,
            ..
        } = trick.complete().unwrap();
        assert_eq!(winner_id, P2);
        assert_eq!(largest_trick_unit_size, 4);
        assert_eq!(points, vec![S_5, S_5, S_5]);
    }

    #[test]
    fn test_play_throw_trick() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4], Number::Four);
        hands.add(P1, vec![H_8, H_8, H_7, H_2]).unwrap();
        hands.add(P2, vec![H_2, S_2, S_2, S_2]).unwrap();
        hands.add(P3, vec![S_2, S_2, S_3, S_4]).unwrap();
        hands.add(P4, vec![S_4, S_4, S_4, S_4]).unwrap();
        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);
        trick
            .play_cards(P1, &mut hands, &[H_8, H_8, H_7, H_2])
            .unwrap();
        trick
            .play_cards(P2, &mut hands, &[H_2, S_2, S_2, S_2])
            .unwrap();
        trick
            .play_cards(P3, &mut hands, &[S_2, S_2, S_3, S_4])
            .unwrap();
        trick
            .play_cards(P4, &mut hands, &[S_4, S_4, S_4, S_4])
            .unwrap();
        let TrickEnded {
            winner: winner_id,
            points,
            largest_trick_unit_size,
            ..
        } = trick.complete().unwrap();
        assert_eq!(largest_trick_unit_size, 2);
        assert_eq!(winner_id, P3);
        assert_eq!(points, vec![]);
    }

    #[test]
    fn test_play_throw_trick_failure() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4], Number::Four);
        hands.add(P1, vec![H_8, H_8, H_7, H_2]).unwrap();
        hands.add(P2, vec![H_2, S_2, S_2, S_2]).unwrap();
        hands.add(P3, vec![S_2, S_2, S_3, S_4]).unwrap();
        hands.add(P4, vec![S_4, S_4, S_4, H_3]).unwrap();
        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);
        trick
            .play_cards(P1, &mut hands, &[H_8, H_8, H_7, H_2])
            .unwrap();
        trick.play_cards(P2, &mut hands, &[H_2]).unwrap();
        trick.play_cards(P3, &mut hands, &[S_3]).unwrap();
        trick.play_cards(P4, &mut hands, &[H_3]).unwrap();
        let TrickEnded {
            winner: winner_id,
            points,
            largest_trick_unit_size,
            failed_throw_size,
            ..
        } = trick.complete().unwrap();
        assert_eq!(largest_trick_unit_size, 1);
        assert_eq!(winner_id, P3);
        assert_eq!(points, vec![]);
        assert_eq!(failed_throw_size, 3);
    }

    #[test]
    fn test_trick_format_basic() {
        let expected_tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: vec![TrickUnit::Repeated {
                count: 3,
                card: S_2,
            }],
        };

        assert_eq!(
            TrickFormat::from_cards(TRUMP, &[S_2, S_2, S_2]).unwrap(),
            expected_tf
        );

        assert!(expected_tf.matches(&[S_2, S_2, S_2]).is_ok());
        assert!(expected_tf.matches(&[S_2, S_2]).is_err());
    }

    #[test]
    fn test_trick_format_tractor() {
        let expected_tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: vec![TrickUnit::Tractor {
                count: 3,
                members: vec![S_2, S_3, S_5],
            }],
        };

        assert_eq!(
            TrickFormat::from_cards(TRUMP, &[S_2, S_2, S_2, S_3, S_3, S_3, S_5, S_5, S_5]).unwrap(),
            expected_tf,
        );
        assert!(expected_tf
            .matches(&[S_2, S_2, S_2, S_3, S_3, S_3, S_5, S_5, S_5])
            .is_ok());
        assert!(expected_tf
            .matches(&[S_3, S_3, S_3, S_5, S_5, S_5, S_6, S_6, S_6])
            .is_ok());
        assert!(expected_tf
            .matches(&[S_2, S_2, S_2, S_3, S_3, S_3, S_6, S_6, S_6])
            .is_err());
    }

    #[test]
    fn test_trick_tractor_throw() {
        let expected_tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: vec![
                TrickUnit::Tractor {
                    count: 2,
                    members: vec![S_3, S_5],
                },
                TrickUnit::Repeated {
                    count: 7,
                    card: S_2,
                },
            ],
        };

        assert_eq!(
            TrickFormat::from_cards(
                TRUMP,
                &[S_2, S_2, S_2, S_2, S_2, S_2, S_2, S_3, S_3, S_5, S_5]
            )
            .unwrap(),
            expected_tf
        );
        assert!(expected_tf
            .matches(&[S_2, S_2, S_2, S_2, S_2, S_2, S_2, S_3, S_3, S_5, S_5])
            .is_ok());
        assert!(expected_tf
            .matches(&[S_8, S_8, S_8, S_8, S_8, S_8, S_8, S_3, S_3, S_5, S_5])
            .is_ok());

        assert!(
            TrickFormat::from_cards(TRUMP, &[S_2, S_2, S_3, S_3, S_5, S_5, S_8, S_8, S_8])
                .unwrap()
                .matches(&[S_2, S_2, S_2, S_2, S_2, S_3, S_3, S_5, S_5])
                .is_ok()
        );
    }

    #[test]
    fn test_trick_simple_throw() {
        let expected_tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: vec![
                TrickUnit::Repeated {
                    count: 1,
                    card: S_3,
                },
                TrickUnit::Repeated {
                    count: 3,
                    card: S_2,
                },
                TrickUnit::Repeated {
                    count: 3,
                    card: S_5,
                },
            ],
        };

        assert_eq!(
            TrickFormat::from_cards(TRUMP, &[S_2, S_2, S_2, S_3, S_5, S_5, S_5]).unwrap(),
            expected_tf
        );

        assert!(expected_tf
            .matches(&[S_5, S_5, S_5, S_3, S_3, S_3, S_2])
            .is_ok());
    }

    #[test]
    fn test_legal_play_pairs() {
        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: vec![TrickUnit::Repeated {
                count: 2,
                card: S_3,
            }],
        };

        let hand = Card::count(vec![S_2, S_2, S_3, S_3, S_5, S_5]);
        assert!(tf.is_legal_play(&hand, &[S_2, S_2]));
        assert!(!tf.is_legal_play(&hand, &[S_2, S_3]));
        assert!(!tf.is_legal_play(&hand, &[S_2, S_3, S_3]));

        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: vec![TrickUnit::Repeated {
                count: 5,
                card: S_3,
            }],
        };
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_3, S_3, S_5]));

        let hand = Card::count(vec![S_2, S_2, S_2, S_2, S_3, S_3, S_5, S_5]);
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_2, S_2, S_5]));

        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: vec![TrickUnit::Tractor {
                count: 2,
                members: vec![S_2, S_3],
            }],
        };
        assert!(!tf.is_legal_play(&hand, &[S_2, S_2, S_2, S_2]));
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_3, S_3]));
        assert!(tf.is_legal_play(&hand, &[S_3, S_3, S_5, S_5]));

        let hand = Card::count(vec![S_2, S_2, S_2, S_2, S_3, S_5, S_5]);
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_2, S_2]));
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_5, S_5]));

        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: vec![
                TrickUnit::Repeated {
                    count: 2,
                    card: S_2,
                },
                TrickUnit::Repeated {
                    count: 1,
                    card: S_3,
                },
            ],
        };
        let hand = Card::count(vec![S_2, S_2, S_2, S_5]);
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_2]));
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_5]));
    }
}
