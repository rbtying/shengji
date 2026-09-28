//! Small card-handling helpers shared by the bot's play and setup logic.

use std::collections::{BTreeMap, HashMap};

use shengji_mechanics::hands::Hands;
use shengji_mechanics::ordered_card::OrderedCard;
use shengji_mechanics::trick::{
    TractorRequirements, TrickDrawPolicy, TrickFormat, TrickUnit, UnitLike,
};
use shengji_mechanics::types::{Card, EffectiveSuit, PlayerID, Trump};

pub(crate) fn points(c: Card) -> usize {
    c.points().unwrap_or(0)
}

pub(crate) fn total_points(cards: &[Card]) -> usize {
    cards.iter().map(|c| points(*c)).sum()
}

pub(crate) fn hand_cards(hands: &Hands, id: PlayerID) -> Vec<Card> {
    match hands.get(id) {
        Ok(h) => Card::cards(h.iter())
            .copied()
            .filter(|c| *c != Card::Unknown)
            .collect(),
        Err(_) => vec![],
    }
}

/// Sort ascending by strength within the effective suit.
pub(crate) fn sort_by_rank(trump: Trump, cards: &mut [Card]) {
    cards.sort_by(|a, b| trump.compare(*a, *b));
}

pub(crate) fn by_suit(trump: Trump, cards: &[Card]) -> BTreeMap<EffectiveSuit, Vec<Card>> {
    let mut m: BTreeMap<EffectiveSuit, Vec<Card>> = BTreeMap::new();
    for c in cards {
        m.entry(trump.effective_suit(*c)).or_default().push(*c);
    }
    for v in m.values_mut() {
        sort_by_rank(trump, v);
    }
    m
}

pub(crate) fn counts(cards: &[Card]) -> HashMap<Card, usize> {
    Card::count(cards.iter().copied())
}

/// `from` minus `take`, as multisets.
pub(crate) fn subtract(from: &[Card], take: &[Card]) -> Vec<Card> {
    let mut c = counts(take);
    let mut out = Vec::with_capacity(from.len());
    for card in from {
        match c.get_mut(card) {
            Some(n) if *n > 0 => *n -= 1,
            _ => out.push(*card),
        }
    }
    out
}

/// Distinct multi-card units (pairs, triples, tractors) formable from cards of one suit.
pub(crate) fn multi_units(
    trump: Trump,
    tractor_requirements: TractorRequirements,
    cards: &[Card],
) -> Vec<Vec<Card>> {
    let mut out: Vec<Vec<Card>> = vec![];
    for decomposition in TrickUnit::find_plays(trump, tractor_requirements, cards.iter().copied())
        .into_iter()
        .take(6)
    {
        for unit in decomposition {
            if unit.size() > 1 {
                let mut cs = unit.cards();
                sort_by_rank(trump, &mut cs);
                if !out.contains(&cs) {
                    out.push(cs);
                }
            }
        }
    }
    out
}

/// Ways to satisfy the led format out of `cards` (all of which are in the led suit, or all trump
/// when ruffing). Tries the exact format first, then the looser decompositions the rules fall back
/// to. Each result is the "required" portion of a follow; it may be shorter than the trick.
pub(crate) fn matchings(
    tf: &TrickFormat,
    cards: &[Card],
    policy: TrickDrawPolicy,
    exact_only: bool,
    limit: usize,
) -> Vec<Vec<Card>> {
    let trump = tf.trump();
    for format in tf.decomposition(policy) {
        let map = OrderedCard::make_map(cards.iter().copied(), trump);
        let res: Vec<Vec<Card>> = UnitLike::check_play(map, format.into_iter(), policy)
            .take(limit)
            .map(|m| {
                m.into_iter()
                    .flat_map(|x| {
                        x.into_iter()
                            .flat_map(|(card, count)| std::iter::repeat_n(card.card, count))
                    })
                    .collect()
            })
            .collect();
        if !res.is_empty() || exact_only {
            return res;
        }
    }
    vec![]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Fill {
    /// Cheapest cards: avoid trump, avoid points, lowest rank.
    Low,
    /// Give away points (partner is winning), otherwise cheap.
    Points,
    /// Highest cards first.
    High,
}

pub(crate) fn pick(trump: Trump, from: &[Card], n: usize, mode: Fill) -> Vec<Card> {
    let mut v = from.to_vec();
    let is_trump = |c: &Card| trump.effective_suit(*c) == EffectiveSuit::Trump;
    match mode {
        Fill::Low => v.sort_by(|a, b| {
            (is_trump(a), points(*a) > 0)
                .cmp(&(is_trump(b), points(*b) > 0))
                .then(trump.compare(*a, *b))
        }),
        Fill::Points => v.sort_by(|a, b| {
            points(*b)
                .cmp(&points(*a))
                .then(is_trump(a).cmp(&is_trump(b)))
                .then(trump.compare(*a, *b))
        }),
        Fill::High => v.sort_by(|a, b| trump.compare(*b, *a)),
    }
    v.truncate(n);
    v
}

/// Sum of rank positions, used to compare two same-sized plays of the same suit.
pub(crate) fn rank_score(trump: Trump, cards: &[Card], suit_cards: &[Card]) -> usize {
    cards
        .iter()
        .map(|c| {
            suit_cards
                .iter()
                .filter(|o| trump.compare(**o, *c) == std::cmp::Ordering::Less)
                .count()
        })
        .sum()
}

/// A card as a person would say it, e.g. "K♤" or "big joker".
pub(crate) fn card_name(c: Card) -> String {
    match c {
        Card::BigJoker => "big joker".to_string(),
        Card::SmallJoker => "small joker".to_string(),
        c => format!("{c:?}")
            .replace('♤', "♠")
            .replace('♡', "♥")
            .replace('♧', "♣")
            .replace('♢', "♦"),
    }
}
