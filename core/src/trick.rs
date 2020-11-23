use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};
use smallvec::{smallvec, SmallVec};
use thiserror::Error;

use crate::hands::{HandError, Hands};
use crate::message::MessageVariant;
use crate::ordered_card::{
    attempt_format_match, subsequent_decomposition_ordering, AdjacentTupleSizes, OrderedCard,
};
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
    #[error("the proposed grouping is invalid")]
    NonMatchingProposal,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TrickDrawPolicy {
    NoProtections,
    LongerTuplesProtected,
}

impl Default for TrickDrawPolicy {
    fn default() -> Self {
        TrickDrawPolicy::NoProtections
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ThrowEvaluationPolicy {
    All,
    Highest,
    TrickUnitLength,
}

impl Default for ThrowEvaluationPolicy {
    fn default() -> Self {
        ThrowEvaluationPolicy::All
    }
}

type Members = SmallVec<[OrderedCard; 3]>;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrickUnit {
    Tractor { count: usize, members: Members },
    Repeated { count: usize, card: OrderedCard },
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
            TrickUnit::Repeated { count, .. } => *count as usize,
            TrickUnit::Tractor {
                count, ref members, ..
            } => (*count as usize) * members.len(),
        }
    }

    pub fn first_card(&self) -> OrderedCard {
        match self {
            TrickUnit::Repeated { card, .. } => *card,
            TrickUnit::Tractor { ref members, .. } => members[0],
        }
    }

    pub fn last_card(&self) -> OrderedCard {
        match self {
            TrickUnit::Repeated { card, .. } => *card,
            TrickUnit::Tractor { ref members, .. } => {
                *members.last().expect("Last card must exist")
            }
        }
    }

    pub fn find_plays(
        trump: Trump,
        iter: impl IntoIterator<Item = Card>,
    ) -> impl IntoIterator<Item = Units> {
        let mut counts = BTreeMap::new();
        let mut original_num_cards = 0;
        for card in iter.into_iter() {
            let card = OrderedCard { card, trump };
            *counts.entry(card).or_insert(0) += 1;
            original_num_cards += 1;
        }

        find_plays_inner(&mut counts, original_num_cards, None, 0)
    }

    pub fn cards(&self) -> SmallVec<[Card; 4]> {
        match self {
            TrickUnit::Tractor {
                count, ref members, ..
            } => members
                .iter()
                .flat_map(|card| (0..*count).map(move |_| card.card))
                .collect(),
            TrickUnit::Repeated { card, count } => (0..*count).map(move |_| card.card).collect(),
        }
    }
}

impl std::fmt::Debug for TrickUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.cards())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrickFormat {
    suit: EffectiveSuit,
    trump: Trump,
    units: Units,
}

impl TrickFormat {
    pub fn trump(&self) -> Trump {
        self.trump
    }

    pub fn size(&self) -> usize {
        self.units.iter().map(|u| u.size()).sum()
    }

    pub fn suit(&self) -> EffectiveSuit {
        self.suit
    }

    pub fn decomposition(&self) -> impl Iterator<Item = SmallVec<[UnitLike; 4]>> {
        let units = self.units.iter().map(UnitLike::from).collect();
        let adj_tuples = self
            .units
            .iter()
            .map(UnitLike::from)
            .map(|u| u.adjacent_tuples)
            .collect();

        // Include the current trick-format, and then the subsequent decomposition if we get that
        // far. Compute the latter lazily, since we usually won't.
        std::iter::once(units).chain(
            std::iter::once_with(|| {
                subsequent_decomposition_ordering(adj_tuples)
                    .into_iter()
                    .map(|requirements| {
                        requirements
                            .into_iter()
                            .map(|adjacent_tuples| UnitLike { adjacent_tuples })
                            .collect()
                    })
            })
            .flatten(),
        )
    }

    pub fn is_legal_play(
        &self,
        hand: &HashMap<Card, usize>,
        proposed: &'_ [Card],
        trick_draw_policy: TrickDrawPolicy,
    ) -> bool {
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
            let available_cards = Card::cards(
                hand.iter()
                    .filter(|(c, _)| self.trump.effective_suit(**c) == self.suit),
            )
            .copied()
            .collect::<Vec<_>>();

            for requirement in self.decomposition() {
                // If it's a match, we're good!
                let play_matches = UnitLike::check_play(
                    self.trump,
                    proposed.iter().copied(),
                    requirement.iter().cloned(),
                    TrickDrawPolicy::NoProtections,
                )
                .0;

                if play_matches {
                    return true;
                }
                // Otherwise, if it could match in the player's hand, it's not OK.
                let hand_can_play = UnitLike::check_play(
                    self.trump,
                    available_cards.iter().copied(),
                    requirement.iter().cloned(),
                    trick_draw_policy,
                )
                .0;
                if hand_can_play {
                    return false;
                }
            }

            // Couldn't meet requirements in either hand or proposed play, so the proposed play is
            // legal.
            true
        }
    }

    pub fn matches(&self, cards: &'_ [Card]) -> Result<Units, TrickError> {
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

        let (found, found_units) = UnitLike::check_play(
            self.trump,
            cards.iter().copied(),
            self.units.iter().map(UnitLike::from),
            TrickDrawPolicy::NoProtections,
        );
        if found {
            debug_assert_eq!(
                self.units
                    .iter()
                    .map(UnitLike::from)
                    .collect::<HashSet<_>>(),
                found_units
                    .iter()
                    .map(UnitLike::from)
                    .collect::<HashSet<_>>()
            );
            Ok(found_units)
        } else {
            Err(TrickError::NonMatchingPlay)
        }
    }

    pub fn from_cards(
        trump: Trump,
        cards: &'_ [Card],
        proposed: Option<&'_ [TrickUnit]>,
    ) -> Result<TrickFormat, TrickError> {
        if cards.is_empty() {
            return Err(TrickError::WrongNumberOfSuits);
        }
        let suit = trump.effective_suit(cards[0]);
        for card in cards {
            if trump.effective_suit(*card) != suit {
                return Err(TrickError::WrongNumberOfSuits);
            }
        }
        let mut possibilities = TrickUnit::find_plays(trump, cards.iter().copied())
            .into_iter()
            .collect::<Vec<Units>>();

        let sort = |mut u: Units| {
            u.sort_by(|a, b| {
                a.size()
                    .cmp(&b.size())
                    .then(a.first_card().cmp(&b.first_card()))
            });
            u
        };

        match proposed {
            Some(proposed) => {
                let proposed = sort(proposed.iter().cloned().collect());
                for possibility in possibilities {
                    if sort(possibility) == proposed {
                        return Ok(TrickFormat {
                            suit,
                            units: proposed,
                            trump,
                        });
                    }
                }
                Err(TrickError::NonMatchingProposal)
            }
            None => {
                possibilities
                    .sort_by_key(|units| units.iter().map(|u| (u.size(), u.is_tractor())).max());
                let units = possibilities.pop().ok_or(TrickError::IllegalPlay)?;
                Ok(TrickFormat {
                    suit,
                    units: sort(units),
                    trump,
                })
            }
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
    /// A parallel array to `played_cards` which contains the units corresponding to played cards
    /// that match the `trick_format`, or `None` if they don't match.
    ///
    /// TODO: remove default deserialization attribute in a few days.
    #[serde(default)]
    played_card_mappings: Vec<Option<Units>>,
    current_winner: Option<PlayerID>,
    trick_format: Option<TrickFormat>,
    trump: Trump,
}

impl Trick {
    pub fn new(trump: Trump, players: impl IntoIterator<Item = PlayerID>) -> Self {
        let player_queue = players.into_iter().collect::<VecDeque<_>>();
        Trick {
            played_cards: Vec::with_capacity(player_queue.len()),
            played_card_mappings: Vec::with_capacity(player_queue.len()),
            current_winner: None,
            trick_format: None,
            player_queue,
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

    pub fn trump(&self) -> Trump {
        self.trump
    }

    pub fn trick_format(&self) -> Option<&'_ TrickFormat> {
        self.trick_format.as_ref()
    }

    ///
    /// Determines whether the player can play the cards.
    ///
    /// Note: this does not account for throw validity, nor is it intended to
    /// catch all illegal plays.
    ///
    pub fn can_play_cards<'a, 'b>(
        &self,
        id: PlayerID,
        hands: &'a Hands,
        cards: &'b [Card],
        trick_draw_policy: TrickDrawPolicy,
    ) -> Result<(), TrickError> {
        hands.contains(id, cards.iter().cloned())?;
        match self.trick_format.as_ref() {
            Some(tf) => {
                if tf.is_legal_play(hands.get(id)?, cards, trick_draw_policy) {
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

    ///
    /// Actually plays the cards, if possible. On error, does not modify any state.
    ///
    /// Note: this does not account throw validity, nor is it intended to catch all illegal plays.
    ///
    pub fn play_cards<'a, 'b>(
        &mut self,
        id: PlayerID,
        hands: &'a mut Hands,
        cards: &'b [Card],
        trick_draw_policy: TrickDrawPolicy,
        throw_eval_policy: ThrowEvaluationPolicy,
        format_hint: Option<&'_ [TrickUnit]>,
    ) -> Result<Vec<MessageVariant>, TrickError> {
        if self.player_queue.front().cloned() != Some(id) {
            return Err(TrickError::OutOfOrder);
        }
        self.can_play_cards(id, hands, cards, trick_draw_policy)?;
        let mut msgs = vec![];
        let mut cards = cards.to_vec();
        cards.sort_by(|a, b| self.trump.compare(*a, *b));

        let (cards, bad_throw_cards, better_player) = if self.trick_format.is_none() {
            let mut tf = TrickFormat::from_cards(self.trump, &cards, format_hint)?;
            let mut invalid = None;
            if tf.units.len() > 1 {
                // This is a throw, let's see if any of the units can be strictly defeated by any
                // other player.
                'search: for player in self.player_queue.iter().skip(1) {
                    let subset_hands = hands.get(*player)?.iter().filter_map(|(card, count)| {
                        if self.trump.effective_suit(*card) == tf.suit {
                            Some((
                                OrderedCard {
                                    card: *card,
                                    trump: self.trump,
                                },
                                *count,
                            ))
                        } else {
                            None
                        }
                    });

                    for unit in &tf.units {
                        match unit {
                            TrickUnit::Repeated { count, card } => {
                                for (c, ct) in subset_hands.clone() {
                                    if ct >= *count && c.cmp_effective(*card) == Ordering::Greater {
                                        invalid = Some((player, unit.clone()));
                                        break 'search;
                                    }
                                }
                            }
                            TrickUnit::Tractor { count, members } => {
                                let in_suit = subset_hands
                                    .clone()
                                    .collect::<BTreeMap<OrderedCard, usize>>();
                                for (c, ct) in in_suit.range(members[1]..) {
                                    let higher_tractors = find_tractors_from_start(
                                        *c,
                                        *ct,
                                        &in_suit,
                                        *count,
                                        members.len(),
                                    );
                                    if !higher_tractors.is_empty() {
                                        invalid = Some((player, unit.clone()));
                                        break 'search;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let (cards, bad_throw_cards, better_player) =
                if let Some((better_player, forced_unit)) = invalid {
                    let forced_cards: Vec<Card> = match forced_unit {
                        TrickUnit::Repeated { card, count } => {
                            (0..count).map(|_| card.card).collect()
                        }
                        TrickUnit::Tractor { ref members, count } => members
                            .iter()
                            .flat_map(|card| (0..count).map(move |_| card.card))
                            .collect(),
                    };

                    tf.units = smallvec![forced_unit];

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

        debug_assert!(self.trick_format.is_some());
        self.played_card_mappings.push(
            self.trick_format
                .as_ref()
                .and_then(|tf| tf.matches(&cards).ok()),
        );

        self.played_cards.push(PlayedCards {
            id,
            cards,
            bad_throw_cards,
            better_player,
        });

        self.current_winner = Self::winner(
            self.trick_format.as_ref(),
            &self.played_cards,
            self.trump,
            throw_eval_policy,
        );

        Ok(msgs)
    }

    /**
     * Takes back cards just played, e.g. in case of dispute.
     */
    pub fn take_back(
        &mut self,
        id: PlayerID,
        hands: &'_ mut Hands,
        throw_eval_policy: ThrowEvaluationPolicy,
    ) -> Result<(), TrickError> {
        if self.played_cards.last().map(|p| p.id) == Some(id) {
            let played = self.played_cards.pop().unwrap();
            self.played_card_mappings.pop();

            hands.add(id, played.cards).unwrap();
            self.player_queue.push_front(id);
            if self.played_cards.is_empty() {
                self.trick_format = None;
            }
            self.current_winner = Self::winner(
                self.trick_format.as_ref(),
                &self.played_cards,
                self.trump,
                throw_eval_policy,
            );
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
        throw_eval_policy: ThrowEvaluationPolicy,
    ) -> Option<PlayerID> {
        match trick_format {
            Some(tf) => {
                let mut winner = (0, tf.units.iter().cloned().collect::<Units>());

                for (idx, pc) in played_cards.iter().enumerate().skip(1) {
                    if let Ok(m) = tf.matches(&pc.cards) {
                        let greater = match throw_eval_policy {
                            ThrowEvaluationPolicy::All => {
                                m.iter().zip(winner.1.iter()).all(|(n, w)| {
                                    trump
                                        .compare_effective(n.first_card().card, w.first_card().card)
                                        == Ordering::Greater
                                })
                            }
                            ThrowEvaluationPolicy::Highest => {
                                let n_max = m
                                    .iter()
                                    .map(|u| u.last_card())
                                    .max()
                                    .expect("trick format cannot be empty");
                                let w_max = winner
                                    .1
                                    .iter()
                                    .map(|u| u.last_card())
                                    .max()
                                    .expect("trick format cannot be empty");
                                trump.compare_effective(n_max.card, w_max.card) == Ordering::Greater
                            }
                            ThrowEvaluationPolicy::TrickUnitLength => {
                                let mut comparisons = m
                                    .iter()
                                    .zip(winner.1.iter())
                                    // Don't worry about single cards if this is a throw, but do
                                    // evaluate them if it isn't!
                                    .filter(|(n, _)| m.len() == 1 || n.size() > 1)
                                    .map(|(n, w)| {
                                        (
                                            n.size(),
                                            trump.compare_effective(
                                                n.first_card().card,
                                                w.first_card().card,
                                            ),
                                        )
                                    })
                                    .collect::<Vec<_>>();
                                // Compare by size first, then try to skip equal-comparisons.
                                comparisons
                                    .sort_by_key(|(s, c)| (-(*s as isize), *c == Ordering::Equal));
                                let mut iter = comparisons.into_iter().map(|(_, c)| c);
                                loop {
                                    match iter.next() {
                                        Some(Ordering::Equal) => {}
                                        Some(Ordering::Greater) => break true,
                                        Some(Ordering::Less) | None => break false,
                                    }
                                }
                            }
                        };
                        if greater {
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct UnitLike {
    adjacent_tuples: AdjacentTupleSizes,
}

impl UnitLike {
    pub fn multi_description(iter: impl Iterator<Item = UnitLike>) -> String {
        let mut counts = BTreeMap::new();
        for u in iter {
            *counts.entry(u.description()).or_default() += 1;
        }
        if counts.len() == 1 {
            let (desc, ct) = counts
                .into_iter()
                .next()
                .expect("only one item in description");
            if ct == 1 {
                format!("a {}", desc)
            } else {
                format!("{} {}", ct, desc)
            }
        } else {
            let mut s =
                counts
                    .into_iter()
                    .fold(String::new(), |mut s, (desc, ct): (String, usize)| {
                        use std::fmt::Write;
                        let _ = write!(s, "{} {}, ", ct, desc);
                        s
                    });
            s.pop();
            s.pop();
            s
        }
    }

    pub fn description(&self) -> String {
        let length = self.adjacent_tuples.len();
        if length == 1 {
            Self::tuple_description(self.adjacent_tuples[0])
        } else if self.rectangular() {
            let count = self.adjacent_tuples[0];

            if length == 2 {
                if count == 2 {
                    "tractor".to_string()
                } else {
                    format!("tractor of {}s", Self::tuple_description(count))
                }
            } else {
                format!("{}-tractor of {}s", length, Self::tuple_description(count))
            }
        } else {
            match length {
                2 => format!(
                    "an adjacent {} and a {}",
                    Self::tuple_description(self.adjacent_tuples[0]),
                    Self::tuple_description(self.adjacent_tuples[1])
                ),
                _ => {
                    let tuples = self.adjacent_tuples[1..length]
                        .iter()
                        .map(|l| Self::tuple_description(*l))
                        .collect::<Vec<_>>();
                    format!(
                        "an adjacent {}, and a {}",
                        tuples.join(", "),
                        Self::tuple_description(self.adjacent_tuples[0])
                    )
                }
            }
        }
    }

    pub fn rectangular(&self) -> bool {
        self.adjacent_tuples
            .iter()
            .all(|v| *v == self.adjacent_tuples[0])
    }

    pub fn num_cards(&self) -> usize {
        self.adjacent_tuples.iter().sum()
    }

    pub fn tuple_description(len: usize) -> String {
        match len {
            1 => "single".to_string(),
            2 => "pair".to_string(),
            3 => "triple".to_string(),
            4 => "quadruple".to_string(),
            5 => "quintuple".to_string(),
            count => format!("{}-tuple", count),
        }
    }

    pub fn check_play(
        trump: Trump,
        iter: impl IntoIterator<Item = Card>,
        units: impl Iterator<Item = UnitLike> + Clone,
        trick_draw_policy: TrickDrawPolicy,
    ) -> (bool, Units) {
        let mut counts = BTreeMap::new();
        for card in iter.into_iter() {
            let card = OrderedCard { card, trump };
            *counts.entry(card).or_insert(0) += 1;
        }
        let (matched, matches) = attempt_format_match(
            &mut counts,
            0,
            units.map(|u| u.adjacent_tuples),
            |counts, matching| match trick_draw_policy {
                TrickDrawPolicy::NoProtections => true,
                TrickDrawPolicy::LongerTuplesProtected => !matching
                    .iter()
                    .any(|(card, count)| counts.get(card).copied().unwrap_or_default() > *count),
            },
        );
        (
            matched,
            matches
                .into_iter()
                .map(|m| {
                    if m.len() == 1 {
                        let (card, count) = m[0];
                        TrickUnit::Repeated { count, card }
                    } else {
                        let min = m.iter().map(|(_, count)| count).min().unwrap();
                        let max = m.iter().map(|(_, count)| count).max().unwrap();
                        assert_eq!(min, max);
                        TrickUnit::Tractor {
                            count: *min,
                            members: m.iter().map(|(card, _)| *card).collect(),
                        }
                    }
                })
                .collect(),
        )
    }
}

impl<'a> From<&'a TrickUnit> for UnitLike {
    fn from(u: &'a TrickUnit) -> Self {
        match u {
            TrickUnit::Tractor { ref members, count } => UnitLike {
                adjacent_tuples: std::iter::repeat(*count)
                    .take(members.len() as usize)
                    .collect(),
            },
            TrickUnit::Repeated { count, .. } => UnitLike {
                adjacent_tuples: smallvec![*count],
            },
        }
    }
}

type Units = SmallVec<[TrickUnit; 4]>;

fn without_trick_unit<T>(
    counts: &mut BTreeMap<OrderedCard, usize>,
    unit: &TrickUnit,
    mut f: impl FnMut(&mut BTreeMap<OrderedCard, usize>) -> T,
) -> T {
    match unit {
        TrickUnit::Repeated { card, count } => {
            let c = counts.get_mut(&card).unwrap();
            if *c == *count {
                counts.remove(&card);
            } else {
                *c -= count;
            }
        }
        TrickUnit::Tractor {
            ref members, count, ..
        } => {
            for card in members {
                let c = counts.get_mut(&card).unwrap();
                if *c == *count {
                    counts.remove(&card);
                } else {
                    *c -= count;
                }
            }
        }
    }

    let res = f(counts);

    match unit {
        TrickUnit::Repeated { card, count } => {
            *counts.entry(*card).or_insert(0) += count;
        }
        TrickUnit::Tractor {
            ref members, count, ..
        } => {
            for card in members {
                *counts.entry(*card).or_insert(0) += count;
            }
        }
    }

    res
}

fn find_tractors_from_start(
    card: OrderedCard,
    count: usize,
    counts: &BTreeMap<OrderedCard, usize>,
    external_min_count: usize,
    min_length: usize,
) -> Units {
    let mut potential_starts = Units::new();

    if count < external_min_count {
        return potential_starts;
    }

    let mut next_cards: SmallVec<[(OrderedCard, Members); 1]> = card
        .successor()
        .into_iter()
        .map(|c| (c, smallvec![card]))
        .collect();
    let mut min_count = count;

    loop {
        let mut next_next_cards = smallvec![];
        for (next_card, mut path) in next_cards {
            let next_count = counts.get(&next_card).copied().unwrap_or(0);
            if next_count >= 2 {
                min_count = min_count.min(next_count);
                path.push(next_card);
                if min_count >= external_min_count && path.len() >= min_length {
                    potential_starts.push(TrickUnit::Tractor {
                        members: path.clone(),
                        count: min_count,
                    });
                }
                next_next_cards
                    .extend(next_card.successor().into_iter().map(|n| (n, path.clone())));
            }
        }
        next_cards = next_next_cards;
        if next_cards.is_empty() {
            break;
        }
    }
    potential_starts
}

fn find_plays_inner(
    counts: &mut BTreeMap<OrderedCard, usize>,
    num_cards: usize,
    min_start: Option<OrderedCard>,
    depth: usize,
) -> SmallVec<[Units; 4]> {
    if num_cards == 0 {
        return smallvec![];
    }

    let mut iter = match min_start {
        Some(c) => counts.range(c..),
        None => counts.range(..),
    };
    // We can skip everything < `min_start` safely, because we pick starts from lowest to highest.
    // The return values are therefore always sorted in reverse `first_card` order.
    let mut potential_starts = Units::new();
    if let Some((card, count)) = iter.next() {
        let new_tractors = find_tractors_from_start(*card, *count, counts, 2, 2);

        let all_consumed = !new_tractors.is_empty()
            && new_tractors.iter().all(|t| match t {
                TrickUnit::Repeated { .. } => unreachable!(),
                TrickUnit::Tractor {
                    ref members,
                    count: width,
                } => members
                    .iter()
                    .all(|c| counts.get(c).copied().unwrap_or(0) == *width),
            });
        potential_starts.extend(new_tractors);

        if !all_consumed {
            potential_starts.push(TrickUnit::Repeated {
                card: *card,
                count: *count,
            });
        }
    }

    if let Some(start) = potential_starts.iter().find(|u| u.size() == num_cards) {
        smallvec![smallvec![start.clone()]]
    } else {
        let mut plays = smallvec![];
        for start in potential_starts {
            without_trick_unit(counts, &start, |subcounts| {
                let sub_plays = find_plays_inner(
                    subcounts,
                    num_cards - start.size(),
                    Some(start.first_card()),
                    depth + 1,
                );
                plays.extend(sub_plays.into_iter().map(|mut play| {
                    play.push(start.clone());
                    play
                }));
            });
        }
        plays
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::iter::FromIterator;

    use smallvec::smallvec;

    use crate::hands::Hands;
    use crate::types::{
        cards::{
            C_10, C_4, C_5, C_6, C_7, C_8, C_K, D_4, D_K, H_2, H_3, H_4, H_5, H_7, H_8, H_9, H_A,
            H_K, S_10, S_2, S_3, S_4, S_5, S_6, S_7, S_8, S_9, S_A, S_J, S_K, S_Q,
        },
        Card, EffectiveSuit, Number, PlayerID, Suit, Trump,
    };

    use super::{
        OrderedCard, ThrowEvaluationPolicy, Trick, TrickDrawPolicy, TrickEnded, TrickError,
        TrickFormat, TrickUnit, UnitLike,
    };

    const TRUMP: Trump = Trump::Standard {
        number: Number::Four,
        suit: Suit::Spades,
    };
    const P1: PlayerID = PlayerID(1);
    const P2: PlayerID = PlayerID(2);
    const P3: PlayerID = PlayerID(3);
    const P4: PlayerID = PlayerID(4);

    macro_rules! oc {
        ($card:expr) => {
            OrderedCard {
                card: $card,
                trump: TRUMP,
            }
        };
        ($card:expr, $trump: expr) => {
            OrderedCard {
                card: $card,
                trump: $trump,
            }
        };
    }

    #[allow(clippy::cognitive_complexity)]
    #[test]
    fn test_play_formats() {
        macro_rules! test_eq {
            ($($x:expr),+; $([$([$($y:expr),+]),+]),+) => {
                let cards = vec![$($x),+];
                let units = TrickUnit::find_plays(TRUMP, cards.iter().copied()).into_iter().collect::<Vec<_>>();
                assert_eq!(
                    units.clone().into_iter().map(|units| {
                        units.into_iter().map(|u| u.cards().into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()
                    }).collect::<HashSet<Vec<Vec<Card>>>>(),
                    HashSet::from_iter(vec![$(vec![$(vec![$($y),+]),+]),+])
                );
                for u in units {
                    let (found, play) = UnitLike::check_play(TRUMP, cards.iter().copied(), u.iter().map(UnitLike::from), TrickDrawPolicy::NoProtections);
                    assert!(found);
                    assert_eq!(
                        u.iter().map(UnitLike::from).collect::<HashSet<_>>(),
                        play.iter().map(UnitLike::from).collect::<HashSet<_>>()
                    );
                }
            }
        }

        test_eq!(H_2, H_3, H_7; [[H_7], [H_3], [H_2]]);
        test_eq!(H_2, H_2, H_2; [[H_2, H_2, H_2]]);
        test_eq!(H_2, H_2, H_3, H_3; [[H_2, H_2, H_3, H_3]]);
        test_eq!(H_2, H_2, H_2, H_3, H_3; [[H_2], [H_2, H_2, H_3, H_3]], [[H_3, H_3], [H_2, H_2, H_2]]);
        test_eq!(H_2, H_2, H_3, H_3, H_3; [[H_3], [H_2, H_2, H_3, H_3]], [[H_3, H_3, H_3], [H_2, H_2]]);
        test_eq!(H_4, H_4, S_4, S_4; [[H_4, H_4, S_4, S_4]]);
        test_eq!(H_4, H_4, S_A, S_A; [[S_A, S_A, H_4, H_4]]);
        test_eq!(S_Q, S_Q, S_K, S_K, S_A; [[S_A], [S_Q, S_Q, S_K, S_K]]);

        test_eq!(H_3, H_3, H_3, H_5, H_5, H_5; [[H_3, H_3, H_3, H_5, H_5, H_5]]);
        test_eq!(H_2, H_2, H_3, H_3, H_3, H_5, H_5, H_5;
            [[H_5, H_5, H_5], [H_3], [H_2, H_2, H_3, H_3]],
            [[H_3, H_3, H_3, H_5, H_5, H_5], [H_2, H_2]],
            [[H_5], [H_3], [H_2, H_2, H_3, H_3, H_5, H_5]]
        );
    }

    #[test]
    fn test_play_singles_trick() {
        let run = |tep: ThrowEvaluationPolicy| {
            let mut hands = Hands::new(vec![P1, P2, P3, P4]);
            hands.add(P1, vec![S_2, S_3, S_5]).unwrap();
            hands.add(P2, vec![S_2, S_3, S_5]).unwrap();
            hands.add(P3, vec![S_2, S_3, S_5]).unwrap();
            hands.add(P4, vec![S_2, S_3, S_5]).unwrap();
            let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);

            trick
                .play_cards(
                    P1,
                    &mut hands,
                    &[S_2],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P2,
                    &mut hands,
                    &[S_5],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P3,
                    &mut hands,
                    &[S_3],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P4,
                    &mut hands,
                    &[S_5],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            let TrickEnded {
                winner: winner_id,
                points,
                largest_trick_unit_size,
                ..
            } = trick.complete().unwrap();
            assert_eq!(winner_id, P2);
            assert_eq!(largest_trick_unit_size, 1);
            assert_eq!(points, vec![S_5, S_5]);
        };
        run(ThrowEvaluationPolicy::All);
        run(ThrowEvaluationPolicy::Highest);
        run(ThrowEvaluationPolicy::TrickUnitLength);
    }

    #[test]
    fn test_play_trump_trick() {
        let run = |tep: ThrowEvaluationPolicy| {
            let mut hands = Hands::new(vec![P1, P2, P3, P4]);
            hands.add(P1, vec![S_2, S_3, S_5]).unwrap();
            hands.add(P2, vec![H_2, H_3, S_4]).unwrap();
            hands.add(P3, vec![S_2, S_3, S_5]).unwrap();
            hands.add(P4, vec![S_2, S_3, S_5]).unwrap();
            let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);

            trick
                .play_cards(
                    P1,
                    &mut hands,
                    &[S_2],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P2,
                    &mut hands,
                    &[S_4],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P3,
                    &mut hands,
                    &[S_3],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P4,
                    &mut hands,
                    &[S_5],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            let TrickEnded {
                winner: winner_id,
                points,
                largest_trick_unit_size,
                ..
            } = trick.complete().unwrap();
            assert_eq!(winner_id, P2);
            assert_eq!(largest_trick_unit_size, 1);
            assert_eq!(points, vec![S_5]);
        };
        run(ThrowEvaluationPolicy::All);
        run(ThrowEvaluationPolicy::Highest);
        run(ThrowEvaluationPolicy::TrickUnitLength);
    }

    #[test]
    fn test_play_pairs_trick() {
        let run = |tep: ThrowEvaluationPolicy| {
            let mut hands = Hands::new(vec![P1, P2, P3, P4]);
            hands.add(P1, vec![S_2, S_2, S_5]).unwrap();
            hands.add(P2, vec![H_2, S_3, S_4]).unwrap();
            hands.add(P3, vec![S_5, S_5, S_5]).unwrap();
            hands.add(P4, vec![S_3, S_4, S_5]).unwrap();
            let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);

            trick
                .play_cards(
                    P1,
                    &mut hands,
                    &[S_2, S_2],
                    TrickDrawPolicy::NoProtections,
                    ThrowEvaluationPolicy::All,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P2,
                    &mut hands,
                    &[S_3, S_4],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P3,
                    &mut hands,
                    &[S_5, S_5],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P4,
                    &mut hands,
                    &[S_3, S_5],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            let TrickEnded {
                winner: winner_id,
                points,
                largest_trick_unit_size,
                ..
            } = trick.complete().unwrap();
            assert_eq!(winner_id, P3);
            assert_eq!(largest_trick_unit_size, 2);
            assert_eq!(points, vec![S_5, S_5, S_5]);
        };
        run(ThrowEvaluationPolicy::All);
        run(ThrowEvaluationPolicy::Highest);
        run(ThrowEvaluationPolicy::TrickUnitLength);
    }

    #[test]
    fn test_play_tractor_trick() {
        let run = |tep: ThrowEvaluationPolicy| {
            let mut hands = Hands::new(vec![P1, P2, P3, P4]);
            hands.add(P1, vec![S_2, S_2, S_3, S_3, S_4]).unwrap();
            hands.add(P2, vec![S_6, S_6, S_7, S_7, S_4]).unwrap();
            hands.add(P3, vec![S_2, S_5, S_5, S_5, S_4]).unwrap();
            hands.add(P4, vec![S_6, S_6, S_6, S_6, S_4]).unwrap();
            let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);

            trick
                .play_cards(
                    P1,
                    &mut hands,
                    &[S_2, S_2, S_3, S_3],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P2,
                    &mut hands,
                    &[S_6, S_6, S_7, S_7],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P3,
                    &mut hands,
                    &[S_2, S_5, S_5, S_5],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P4,
                    &mut hands,
                    &[S_6, S_6, S_6, S_6],
                    TrickDrawPolicy::NoProtections,
                    tep,
                    None,
                )
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
        };
        run(ThrowEvaluationPolicy::All);
        run(ThrowEvaluationPolicy::Highest);
        run(ThrowEvaluationPolicy::TrickUnitLength);
    }

    #[test]
    fn test_play_throw_trick() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4]);
        hands.add(P1, vec![H_8, H_8, H_7, H_2]).unwrap();
        hands.add(P2, vec![H_2, S_2, S_2, S_2]).unwrap();
        hands.add(P3, vec![S_2, S_2, S_3, S_4]).unwrap();
        hands.add(P4, vec![S_4, S_4, S_4, S_4]).unwrap();
        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);
        trick
            .play_cards(
                P1,
                &mut hands,
                &[H_8, H_8, H_7, H_2],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P2,
                &mut hands,
                &[H_2, S_2, S_2, S_2],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P3,
                &mut hands,
                &[S_2, S_2, S_3, S_4],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P4,
                &mut hands,
                &[S_4, S_4, S_4, S_4],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
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
        let mut hands = Hands::new(vec![P1, P2, P3, P4]);
        hands.add(P1, vec![H_8, H_8, H_7, H_2]).unwrap();
        hands.add(P2, vec![H_2, S_2, S_2, S_2]).unwrap();
        hands.add(P3, vec![S_2, S_2, S_3, S_4]).unwrap();
        hands.add(P4, vec![S_4, S_4, S_4, H_3]).unwrap();
        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);
        trick
            .play_cards(
                P1,
                &mut hands,
                &[H_8, H_8, H_7, H_2],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P2,
                &mut hands,
                &[H_2],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P3,
                &mut hands,
                &[S_3],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P4,
                &mut hands,
                &[H_3],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
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
    fn test_play_throw_tractor_extra_cards() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4]);
        hands.add(P1, vec![S_Q, S_Q, S_K, S_K, S_A]).unwrap();
        hands.add(P2, vec![S_2, S_3, S_3, S_5, H_3]).unwrap();
        hands.add(P3, vec![S_A, S_A, H_3, H_3, H_3]).unwrap();
        hands.add(P4, vec![H_3, H_3, H_3, H_3, H_3]).unwrap();
        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);
        trick
            .play_cards(
                P1,
                &mut hands,
                &[S_Q, S_Q, S_K, S_K, S_A],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P2,
                &mut hands,
                &[S_2, S_3, S_3, S_5, H_3],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P3,
                &mut hands,
                &[S_A, S_A, H_3, H_3, H_3],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P4,
                &mut hands,
                &[H_3, H_3, H_3, H_3, H_3],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        let TrickEnded {
            winner: winner_id,
            points,
            largest_trick_unit_size,
            failed_throw_size,
            ..
        } = trick.complete().unwrap();
        assert_eq!(largest_trick_unit_size, 4);
        assert_eq!(winner_id, P1);
        assert_eq!(
            points.into_iter().flat_map(|c| c.points()).sum::<usize>(),
            25
        );
        assert_eq!(failed_throw_size, 0);
    }

    #[test]
    fn test_trick_format_basic() {
        let expected_tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: smallvec![TrickUnit::Repeated {
                count: 3,
                card: oc!(S_2),
            }],
        };

        assert_eq!(
            TrickFormat::from_cards(TRUMP, &[S_2, S_2, S_2], None).unwrap(),
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
            units: smallvec![TrickUnit::Tractor {
                count: 3,
                members: smallvec![oc!(S_2), oc!(S_3), oc!(S_5)],
            }],
        };

        assert_eq!(
            TrickFormat::from_cards(TRUMP, &[S_2, S_2, S_2, S_3, S_3, S_3, S_5, S_5, S_5], None)
                .unwrap(),
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
            units: smallvec![
                TrickUnit::Tractor {
                    count: 2,
                    members: smallvec![oc!(S_3), oc!(S_5)],
                },
                TrickUnit::Repeated {
                    count: 7,
                    card: oc!(S_2),
                },
            ],
        };

        assert_eq!(
            TrickFormat::from_cards(
                TRUMP,
                &[S_2, S_2, S_2, S_2, S_2, S_2, S_2, S_3, S_3, S_5, S_5],
                None
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

        assert!(TrickFormat::from_cards(
            TRUMP,
            &[S_2, S_2, S_3, S_3, S_5, S_5, S_8, S_8, S_8],
            None
        )
        .unwrap()
        .matches(&[S_2, S_2, S_2, S_2, S_2, S_3, S_3, S_5, S_5])
        .is_ok());
    }

    #[test]
    fn test_trick_simple_throw() {
        let expected_tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: smallvec![
                TrickUnit::Repeated {
                    count: 1,
                    card: oc!(S_3),
                },
                TrickUnit::Repeated {
                    count: 3,
                    card: oc!(S_2),
                },
                TrickUnit::Repeated {
                    count: 3,
                    card: oc!(S_5),
                },
            ],
        };

        assert_eq!(
            TrickFormat::from_cards(TRUMP, &[S_2, S_2, S_2, S_3, S_5, S_5, S_5], None).unwrap(),
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
            units: smallvec![TrickUnit::Repeated {
                count: 2,
                card: oc!(S_3),
            }],
        };

        let hand = Card::count(vec![S_2, S_2, S_3, S_3, S_5, S_5]);
        assert!(tf.is_legal_play(&hand, &[S_2, S_2], TrickDrawPolicy::NoProtections));
        assert!(!tf.is_legal_play(&hand, &[S_2, S_3], TrickDrawPolicy::NoProtections));
        assert!(!tf.is_legal_play(&hand, &[S_2, S_3, S_3], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(&hand, &[S_2, S_2], TrickDrawPolicy::NoProtections));
        assert!(!tf.is_legal_play(&hand, &[S_2, S_3], TrickDrawPolicy::NoProtections));
        assert!(!tf.is_legal_play(&hand, &[S_2, S_3, S_3], TrickDrawPolicy::NoProtections));

        // Check that we don't break longer tuples if that's not required
        let hand = Card::count(vec![S_2, S_2, S_2, S_3, S_5]);
        assert!(tf.is_legal_play(&hand, &[S_3, S_5], TrickDrawPolicy::LongerTuplesProtected));
        assert!(!tf.is_legal_play(&hand, &[S_3, S_5], TrickDrawPolicy::NoProtections));

        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: smallvec![TrickUnit::Repeated {
                count: 3,
                card: oc!(S_3),
            }],
        };

        let hand = Card::count(vec![S_2, S_2, S_3, S_3, S_5, S_5]);
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_5], TrickDrawPolicy::NoProtections,));
        assert!(!tf.is_legal_play(&hand, &[S_2, S_3, S_5], TrickDrawPolicy::NoProtections,));
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_5], TrickDrawPolicy::NoProtections,));
        assert!(!tf.is_legal_play(&hand, &[S_2, S_3, S_5], TrickDrawPolicy::NoProtections,));

        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: smallvec![TrickUnit::Repeated {
                count: 5,
                card: oc!(S_3),
            }],
        };
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_3, S_3, S_5],
            TrickDrawPolicy::NoProtections
        ));
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_3, S_3, S_5],
            TrickDrawPolicy::NoProtections
        ));

        let hand = Card::count(vec![S_2, S_2, S_2, S_2, S_3, S_3, S_5, S_5]);
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_2, S_2, S_5],
            TrickDrawPolicy::NoProtections
        ));
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_2, S_2, S_5],
            TrickDrawPolicy::NoProtections
        ));

        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: smallvec![TrickUnit::Tractor {
                count: 2,
                members: smallvec![oc!(S_2), oc!(S_3)],
            }],
        };
        assert!(!tf.is_legal_play(&hand, &[S_2, S_2, S_2, S_2], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_3, S_3], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(&hand, &[S_3, S_3, S_5, S_5], TrickDrawPolicy::NoProtections));
        assert!(!tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_2, S_2],
            TrickDrawPolicy::LongerTuplesProtected
        ));
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_3, S_3],
            TrickDrawPolicy::LongerTuplesProtected
        ));
        assert!(tf.is_legal_play(
            &hand,
            &[S_3, S_3, S_5, S_5],
            TrickDrawPolicy::LongerTuplesProtected
        ));

        let hand = Card::count(vec![S_2, S_2, S_2, S_2, S_3, S_5, S_5]);
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_2, S_2], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_5, S_5], TrickDrawPolicy::NoProtections));
        assert!(!tf.is_legal_play(&hand, &[S_2, S_2, S_5, S_3], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_2, S_2],
            TrickDrawPolicy::LongerTuplesProtected
        ));
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_5, S_5],
            TrickDrawPolicy::LongerTuplesProtected
        ));
        // This play is tenuously legal, since the 2222 is protected by the 355 is not, and the
        // trick-format is 2233. Normally we would expect that the 2233 is required, but the player
        // has decided to break the 22 but *not* play the 55.
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_5, S_3],
            TrickDrawPolicy::LongerTuplesProtected
        ));

        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: smallvec![
                TrickUnit::Repeated {
                    count: 2,
                    card: oc!(S_2),
                },
                TrickUnit::Repeated {
                    count: 1,
                    card: oc!(S_3),
                },
            ],
        };
        let hand = Card::count(vec![S_2, S_2, S_2, S_5]);
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_2], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(&hand, &[S_2, S_2, S_5], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_2],
            TrickDrawPolicy::LongerTuplesProtected
        ));
        assert!(tf.is_legal_play(
            &hand,
            &[S_2, S_2, S_5],
            TrickDrawPolicy::LongerTuplesProtected
        ));
    }

    #[test]
    fn test_protected_tuple() {
        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: smallvec![TrickUnit::Repeated {
                card: oc!(S_3),
                count: 3,
            },],
        };
        let hand = Card::count(vec![S_2, S_2, S_2, S_2, S_5, S_6, S_7, S_8]);
        assert!(!tf.is_legal_play(&hand, &[S_6, S_7, S_8], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(
            &hand,
            &[S_6, S_7, S_8],
            TrickDrawPolicy::LongerTuplesProtected
        ));
        let hand = Card::count(vec![S_2, S_2, S_2, S_2, S_5, S_5, S_6, S_7, S_8]);
        assert!(!tf.is_legal_play(&hand, &[S_5, S_5, S_6], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(
            &hand,
            &[S_5, S_5, S_6],
            TrickDrawPolicy::LongerTuplesProtected
        ));
        assert!(!tf.is_legal_play(
            &hand,
            &[S_6, S_7, S_8],
            TrickDrawPolicy::LongerTuplesProtected
        ));
    }

    #[test]
    fn test_protected_wider_tractor() {
        let tf = TrickFormat {
            suit: EffectiveSuit::Trump,
            trump: TRUMP,
            units: smallvec![TrickUnit::Tractor {
                members: smallvec![oc!(S_6), oc!(S_7)],
                count: 2,
            },],
        };
        let hand = Card::count(vec![S_2, S_2, S_2, S_3, S_3, S_3, S_5, S_6, S_7, S_8]);
        assert!(!tf.is_legal_play(&hand, &[S_5, S_6, S_7, S_8], TrickDrawPolicy::NoProtections));
        assert!(tf.is_legal_play(
            &hand,
            &[S_5, S_6, S_7, S_8],
            TrickDrawPolicy::LongerTuplesProtected
        ));
    }

    #[test]
    fn test_protected_tractor_triple() {
        const HEART_TRUMP: Trump = Trump::Standard {
            number: Number::Four,
            suit: Suit::Hearts,
        };
        let tf = TrickFormat {
            suit: EffectiveSuit::Spades,
            trump: HEART_TRUMP,
            units: smallvec![
                TrickUnit::Tractor {
                    members: smallvec![oc!(S_9, HEART_TRUMP), oc!(S_9, HEART_TRUMP)],
                    count: 2,
                },
                TrickUnit::Repeated {
                    card: oc!(S_K, HEART_TRUMP),
                    count: 1,
                }
            ],
        };
        let hand = Card::count(vec![S_3, S_5, S_10, S_J, S_Q, S_6, S_8, S_8, S_8]);
        assert!(!tf.is_legal_play(
            &hand,
            &[S_3, S_5, S_10, S_J, S_Q],
            TrickDrawPolicy::NoProtections
        ));
        assert!(tf.is_legal_play(
            &hand,
            &[S_3, S_6, S_8, S_8, S_8],
            TrickDrawPolicy::NoProtections
        ));
        assert!(tf.is_legal_play(
            &hand,
            &[S_3, S_5, S_10, S_J, S_Q],
            TrickDrawPolicy::LongerTuplesProtected
        ));
    }

    #[test]
    fn test_play_throw_tractor_with_other_tractor_in_game() {
        let trump = Trump::Standard {
            number: Number::Four,
            suit: Suit::Hearts,
        };

        let mut hands = Hands::new(vec![P1, P2, P3, P4]);
        let p2_hand = vec![H_2, H_2, H_3, H_A, H_3];
        let p1_hand = vec![S_Q, S_Q, S_K, S_K, S_A];
        let p3_hand = vec![S_A, S_A, S_3, S_3, S_3];
        let p4_hand = vec![S_3, S_3, S_3, S_3, S_3];

        hands.add(P1, p1_hand.clone()).unwrap();
        hands.add(P2, p2_hand.clone()).unwrap();
        hands.add(P3, p3_hand.clone()).unwrap();
        hands.add(P4, p4_hand.clone()).unwrap();
        let mut trick = Trick::new(trump, vec![P1, P2, P3, P4]);
        trick
            .play_cards(
                P1,
                &mut hands,
                &p1_hand,
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P2,
                &mut hands,
                &p2_hand,
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P3,
                &mut hands,
                &p3_hand,
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P4,
                &mut hands,
                &p4_hand,
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        let TrickEnded {
            winner: winner_id,
            points,
            largest_trick_unit_size,
            failed_throw_size,
            ..
        } = trick.complete().unwrap();
        assert_eq!(largest_trick_unit_size, 4);
        assert_eq!(winner_id, P2);
        assert_eq!(points, vec![S_K, S_K]);
        assert_eq!(failed_throw_size, 0);
    }

    #[test]
    fn test_long_tractor_decomposition_draws_pairs() {
        let trump = Trump::Standard {
            number: Number::King,
            suit: Suit::Spades,
        };

        let p1_hand = vec![S_7, S_7, S_8, S_8, S_9, S_9, C_4, C_4];
        let p2_hand = vec![S_4, S_10, S_A, H_K, D_K, C_K, S_K, S_K];
        let p3_hand = vec![S_2, S_6, S_J, S_Q, H_K, C_K, C_10, Card::SmallJoker];
        let p4_hand = vec![C_4, C_6, C_7, S_3, S_3, Card::BigJoker, C_5, C_8];

        for policy in &[
            TrickDrawPolicy::NoProtections,
            TrickDrawPolicy::LongerTuplesProtected,
        ] {
            let mut hands = Hands::new(vec![P1, P2, P3, P4]);

            hands.add(P1, p1_hand.clone()).unwrap();
            hands.add(P2, p2_hand.clone()).unwrap();
            hands.add(P3, p3_hand.clone()).unwrap();
            hands.add(P4, p4_hand.clone()).unwrap();

            let mut trick = Trick::new(trump, vec![P1, P2, P3, P4]);

            trick
                .play_cards(
                    P1,
                    &mut hands,
                    &[S_7, S_7, S_8, S_8, S_9, S_9],
                    *policy,
                    ThrowEvaluationPolicy::All,
                    None,
                )
                .unwrap();
            // This play should not succeed, because P2 also has S_K, S_K which is a pair.
            if let Err(TrickError::IllegalPlay) = trick.play_cards(
                P2,
                &mut hands,
                &[S_4, S_10, S_A, H_K, D_K, C_K],
                *policy,
                ThrowEvaluationPolicy::All,
                None,
            ) {
                trick
                    .play_cards(
                        P2,
                        &mut hands,
                        &[S_4, S_10, S_A, H_K, S_K, S_K],
                        *policy,
                        ThrowEvaluationPolicy::All,
                        None,
                    )
                    .unwrap();
            } else {
                panic!("Expected play to be illegal, but it wasn't")
            }
            trick
                .play_cards(
                    P3,
                    &mut hands,
                    &[S_2, S_6, S_J, S_Q, H_K, C_K],
                    *policy,
                    ThrowEvaluationPolicy::All,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P4,
                    &mut hands,
                    &[C_4, C_6, C_7, S_3, S_3, Card::BigJoker],
                    *policy,
                    ThrowEvaluationPolicy::All,
                    None,
                )
                .unwrap();
        }
    }

    #[test]
    fn test_throw_evaluation_policy_highest_card() {
        let trump = Trump::Standard {
            number: Number::King,
            suit: Suit::Spades,
        };

        let p1_hand = vec![C_4, C_6];
        let p2_hand = vec![S_2, S_3];
        let p3_hand = vec![S_3, S_4];
        let p4_hand = vec![S_2, Card::BigJoker];

        let run = |policy: ThrowEvaluationPolicy| {
            let mut hands = Hands::new(vec![P1, P2, P3, P4]);

            hands.add(P1, p1_hand.clone()).unwrap();
            hands.add(P2, p2_hand.clone()).unwrap();
            hands.add(P3, p3_hand.clone()).unwrap();
            hands.add(P4, p4_hand.clone()).unwrap();

            let mut trick = Trick::new(trump, vec![P1, P2, P3, P4]);

            trick
                .play_cards(
                    P1,
                    &mut hands,
                    &p1_hand,
                    TrickDrawPolicy::NoProtections,
                    policy,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P2,
                    &mut hands,
                    &p2_hand,
                    TrickDrawPolicy::NoProtections,
                    policy,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P3,
                    &mut hands,
                    &p3_hand,
                    TrickDrawPolicy::NoProtections,
                    policy,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P4,
                    &mut hands,
                    &p4_hand,
                    TrickDrawPolicy::NoProtections,
                    policy,
                    None,
                )
                .unwrap();
            let TrickEnded { winner, .. } = trick.complete().unwrap();
            winner
        };

        // P4 beats P3's highest card, but one of P3's cards beats P4's lowest card.

        // In the "all" case, P3 retains the "winner" status.
        assert_eq!(run(ThrowEvaluationPolicy::All), P3);
        // In the "highest" case, P4 wins because P4 played a higher card.
        assert_eq!(run(ThrowEvaluationPolicy::Highest), P4);
    }

    #[test]
    fn test_throw_evaluation_policy_trick_unit_length() {
        let trump = Trump::Standard {
            number: Number::Two,
            suit: Suit::Spades,
        };

        let p1_hand = vec![H_A, H_K, H_K, H_K, H_9, H_9];
        let p2_hand = vec![S_5, S_5, S_5, S_Q, S_Q, S_A];
        let p3_hand = vec![S_5, S_5, S_5, S_A, S_A, S_4];
        let p4_hand = vec![S_4, S_4, S_4, S_3, S_3, Card::BigJoker];

        let run = |policy: ThrowEvaluationPolicy| {
            let mut hands = Hands::new(vec![P1, P2, P3, P4]);

            hands.add(P1, p1_hand.clone()).unwrap();
            hands.add(P2, p2_hand.clone()).unwrap();
            hands.add(P3, p3_hand.clone()).unwrap();
            hands.add(P4, p4_hand.clone()).unwrap();

            let mut trick = Trick::new(trump, vec![P1, P2, P3, P4]);

            trick
                .play_cards(
                    P1,
                    &mut hands,
                    &p1_hand,
                    TrickDrawPolicy::NoProtections,
                    policy,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P2,
                    &mut hands,
                    &p2_hand,
                    TrickDrawPolicy::NoProtections,
                    policy,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P3,
                    &mut hands,
                    &p3_hand,
                    TrickDrawPolicy::NoProtections,
                    policy,
                    None,
                )
                .unwrap();
            trick
                .play_cards(
                    P4,
                    &mut hands,
                    &p4_hand,
                    TrickDrawPolicy::NoProtections,
                    policy,
                    None,
                )
                .unwrap();
            let TrickEnded { winner, .. } = trick.complete().unwrap();
            winner
        };

        // In the "all" case, P2 retains the "winner" status, since there are no strictly higher
        // plays
        assert_eq!(run(ThrowEvaluationPolicy::All), P2);
        // In the "highest" case, P4 wins because P4 played the highest card (and matched format)
        assert_eq!(run(ThrowEvaluationPolicy::Highest), P4);
        // In the "trick unit length" case, P3 wins because P3 matched-or-beat P2's longest tuples.
        assert_eq!(run(ThrowEvaluationPolicy::TrickUnitLength), P3);
    }

    #[test]
    fn test_throw_of_trump_rank_in_trump() {
        let mut hands = Hands::new(vec![P1, P2, P3, P4]);
        hands.add(P1, vec![H_4, S_4]).unwrap();
        hands.add(P2, vec![D_4, S_2]).unwrap();
        hands.add(P3, vec![S_3, S_3]).unwrap();
        hands.add(P4, vec![S_3, S_3]).unwrap();

        let mut trick = Trick::new(TRUMP, vec![P1, P2, P3, P4]);
        trick
            .play_cards(
                P1,
                &mut hands,
                &[H_4, S_4],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P2,
                &mut hands,
                &[D_4, S_2],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P3,
                &mut hands,
                &[S_3, S_3],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        trick
            .play_cards(
                P4,
                &mut hands,
                &[S_3, S_3],
                TrickDrawPolicy::NoProtections,
                ThrowEvaluationPolicy::All,
                None,
            )
            .unwrap();
        let TrickEnded {
            winner: winner_id,
            points,
            largest_trick_unit_size,
            ..
        } = trick.complete().unwrap();
        assert_eq!(largest_trick_unit_size, 1);
        assert_eq!(winner_id, P1);
        assert_eq!(points, vec![]);
    }
}
