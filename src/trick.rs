use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
            let counts = Card::count(proposed.into_iter().cloned());
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
                        .into_iter()
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

        let counts = Card::count(cards.into_iter().cloned());
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
            let mut counts = Card::count(cards.into_iter().cloned());
            let mut units = vec![];
            loop {
                let mut tractors = find_tractors(trump, &counts);

                // If the tractor is shorter than one of its segments alone, don't
                // include it.
                tractors.retain(|(tractor, _, size)| {
                    *size >= tractor.iter().map(|cc| counts[&cc]).max().unwrap()
                });

                match tractors.pop() {
                    Some((tractor, count, _)) => {
                        // reduce the counts appropriately
                        for card in &tractor {
                            *counts.get_mut(card).unwrap() -= count;
                        }
                        units.push(TrickUnit::Tractor {
                            count,
                            members: tractor,
                        });
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
    id: PlayerID,
    cards: Vec<Card>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trick {
    player_queue: VecDeque<PlayerID>,
    played_cards: Vec<PlayedCards>,
    trick_format: Option<TrickFormat>,
    trump: Trump,
}

impl Trick {
    pub fn new(trump: Trump, players: impl IntoIterator<Item = PlayerID>) -> Self {
        Trick {
            player_queue: players.into_iter().collect(),
            played_cards: vec![],
            trick_format: None,
            trump,
        }
    }

    pub fn next_player(&self) -> Option<PlayerID> {
        self.player_queue.front().cloned()
    }

    pub fn player_queue(&self) -> impl Iterator<Item = PlayerID> + '_ {
        self.player_queue.iter().map(|id| *id)
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
    ) -> Result<(), TrickError> {
        self.can_play_cards(id, hands, cards)?;
        if self.trick_format.is_none() {
            self.trick_format = Some(TrickFormat::from_cards(self.trump, cards)?);
        }
        self.player_queue.pop_front();
        self.played_cards.push(PlayedCards {
            id,
            cards: cards.iter().cloned().collect(),
        });
        hands.remove(id, cards.iter().cloned())?;
        Ok(())
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
            Ok(())
        } else {
            Err(TrickError::OutOfOrder)
        }
    }

    /**
     * Completes the trick and determines the winner. Returns the point cards that the winner won.
     */
    pub fn complete(&self) -> Result<(PlayerID, Vec<Card>, usize), TrickError> {
        if !self.player_queue.is_empty() || self.played_cards.is_empty() {
            return Err(TrickError::OutOfOrder);
        }
        if let Some(tf) = self.trick_format.as_ref() {
            let all_card_points = self
                .played_cards
                .iter()
                .flat_map(|pc| pc.cards.iter().filter(|c| c.points().is_some()).map(|c| *c))
                .collect::<Vec<Card>>();

            let mut winner = (0, tf.units.clone());

            for idx in 1..self.played_cards.len() {
                if let Ok(m) = tf.matches(&self.played_cards[idx].cards) {
                    if m.iter().zip(winner.1.iter()).all(|(n, w)| {
                        self.trump.compare(n.first_card(), w.first_card()) == Ordering::Greater
                    }) {
                        winner = (idx, m);
                    }
                }
            }

            Ok((
                self.played_cards[winner.0].id,
                all_card_points,
                tf.units.iter().map(|u| u.size()).max().unwrap_or(0),
            ))
        } else {
            Err(TrickError::OutOfOrder)
        }
    }
}

pub fn find_tractors(
    trump: Trump,
    counts: &HashMap<Card, usize>,
) -> Vec<(Vec<Card>, usize, usize)> {
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
                tractors.push((p, min_count, size));
            }
        }
    }

    // Sort the tractors by the number of available cards
    tractors.sort_by_key(|(_, _, size)| *size);
    tractors
}

pub fn check_format_matches(
    trump: Trump,
    units: &'_ [TrickUnit],
    mut counts: HashMap<Card, usize>,
) -> bool {
    let matched = check_format_matches_inner(trump, units, &mut counts, &mut vec![]);
    matched
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
                available_tractors.retain(|(t, ct, _)| t.len() == members.len() && *ct == *count);
                for (tractor, count, _) in available_tractors {
                    for card in &tractor {
                        *counts.get_mut(card).unwrap() -= count;
                    }
                    if check_format_matches_inner(
                        trump,
                        &units[..units.len() - 1],
                        counts,
                        allocations,
                    ) {
                        if !allocations.is_empty() {
                            allocations[units.len() - 1] = TrickUnit::Tractor {
                                members: tractor.clone(),
                                count: count,
                            };
                        }
                        return true;
                    } else {
                        for card in &tractor {
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
    use super::{Trick, TrickFormat, TrickUnit};

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
        let (winner_id, points, multiplier) = trick.complete().unwrap();
        assert_eq!(winner_id, P2);
        assert_eq!(multiplier, 1);
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
        let (winner_id, points, multiplier) = trick.complete().unwrap();
        assert_eq!(winner_id, P2);
        assert_eq!(multiplier, 1);
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
        let (winner_id, points, multiplier) = trick.complete().unwrap();
        assert_eq!(winner_id, P3);
        assert_eq!(multiplier, 2);
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
        let (winner_id, points, multiplier) = trick.complete().unwrap();
        assert_eq!(winner_id, P2);
        assert_eq!(multiplier, 4);
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
        let (winner_id, points, multiplier) = trick.complete().unwrap();
        assert_eq!(multiplier, 2);
        assert_eq!(winner_id, P3);
        assert_eq!(points, vec![]);
    }

    #[test]
    fn test_play_throw_trick_take_back() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4], Number::Four);
        hands.add(P1, vec![H_8, H_8, H_7, H_2]).unwrap();
        hands.add(P2, vec![H_2, S_2, S_2, S_2]).unwrap();
        hands.add(P3, vec![S_2, S_2, S_3, S_4]).unwrap();
        hands.add(P4, vec![S_4, S_4, S_4, H_3]).unwrap();
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
        trick.take_back(P3, &mut hands).unwrap();
        trick.take_back(P2, &mut hands).unwrap();
        trick.take_back(P1, &mut hands).unwrap();
        trick.play_cards(P1, &mut hands, &[H_2]).unwrap();
        trick.play_cards(P2, &mut hands, &[H_2]).unwrap();
        trick.play_cards(P3, &mut hands, &[S_3]).unwrap();
        trick.play_cards(P4, &mut hands, &[H_3]).unwrap();
        let (winner_id, points, multiplier) = trick.complete().unwrap();
        assert_eq!(multiplier, 1);
        assert_eq!(winner_id, P3);
        assert_eq!(points, vec![]);
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
