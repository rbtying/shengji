//! A sharper, still fast, play policy for simulated rollouts.
//!
//! Rollouts play out a fully dealt world, so this policy may look at every hand in it. It plays
//! the way a solid player would with that knowledge in the current trick only: lead cards that
//! can't be beaten, take tricks as cheaply as possible, trump in when points are at stake, and
//! give partners points only once the trick is safely theirs.

use rand::rngs::StdRng;
use rand::Rng;

use shengji_mechanics::trick::PlayCards;
use shengji_mechanics::types::{Card, EffectiveSuit, PlayerID};

use super::heuristics::{
    by_suit, hand_cards, matchings, multi_units, pick, points, rank_score, subtract, total_points,
    Fill,
};
use super::play::{policy_play, Sides};
use crate::game_state::play_phase::PlayPhase;

/// Whether playing `cards` would make `id` the current winner of the trick.
fn would_win(p: &PlayPhase, id: PlayerID, cards: &[Card]) -> bool {
    let mut trick = p.trick.clone();
    let mut hands = p.hands.clone();
    trick
        .play_cards(PlayCards {
            id,
            hands: &mut hands,
            cards,
            trick_draw_policy: p.propagated.trick_draw_policy,
            throw_eval_policy: p.propagated.throw_evaluation_policy,
            format_hint: None,
            hide_throw_halting_player: false,
            tractor_requirements: p.propagated.tractor_requirements,
            bomb_policy: p.propagated.bomb_policy,
            compound_formats: p.propagated.compound_formats.clone(),
        })
        .is_ok()
        && trick.current_winner() == Some(id)
}

pub(crate) fn smart_play(
    p: &PlayPhase,
    id: PlayerID,
    sides: &Sides,
    rng: &mut StdRng,
) -> Vec<Card> {
    let trump = p.trump;
    let hand = hand_cards(&p.hands, id);
    let players: Vec<PlayerID> = p.propagated.players.iter().map(|pl| pl.id).collect();
    let opponents: Vec<Vec<Card>> = players
        .iter()
        .filter(|o| **o != id && !sides.same(**o, id))
        .map(|o| hand_cards(&p.hands, *o))
        .collect();
    let others: Vec<Vec<Card>> = players
        .iter()
        .filter(|o| **o != id)
        .map(|o| hand_cards(&p.hands, *o))
        .collect();
    let in_suit_of = |cards: &[Card], s: EffectiveSuit| -> Vec<Card> {
        cards
            .iter()
            .copied()
            .filter(|c| trump.effective_suit(*c) == s)
            .collect()
    };

    let tf = match p.trick.trick_format() {
        None => return lead(p, &hand, &opponents, &others, rng),
        Some(tf) => tf,
    };
    let suit = tf.suit();
    let size = tf.size();
    let policy = p.propagated.trick_draw_policy;
    let played = p.trick.played_cards().len();
    let last = played + 1 >= players.len();
    let partner_winning = p
        .trick
        .current_winner()
        .map(|w| sides.same(w, id))
        .unwrap_or(false);
    let trick_points: usize = p
        .trick
        .played_cards()
        .iter()
        .map(|pc| total_points(&pc.cards))
        .sum();
    // Give points to a partner only when nobody can take the trick back.
    let feed = partner_winning && last;
    let in_suit = in_suit_of(&hand, suit);
    let rest_of_hand = subtract(&hand, &in_suit);

    let complete = |m: Vec<Card>, pool: &[Card], fill: Fill| -> Vec<Card> {
        let rest = subtract(pool, &m);
        let mut c = m;
        let need = size.saturating_sub(c.len());
        c.extend(pick(trump, &rest, need, fill));
        c
    };

    if in_suit.len() >= size {
        let ms = matchings(tf, &in_suit, policy, false, 8);
        if ms.is_empty() {
            return policy_play(p, id, sides, rng);
        }
        if partner_winning {
            let fill = if feed { Fill::Points } else { Fill::Low };
            let options: Vec<Vec<Card>> = ms
                .into_iter()
                .map(|m| complete(m, &in_suit, fill))
                .collect();
            return if feed {
                options
                    .into_iter()
                    .max_by_key(|c| (total_points(c), usize::MAX - rank_score(trump, c, &in_suit)))
                    .unwrap_or_default()
            } else {
                options
                    .into_iter()
                    .min_by_key(|c| (total_points(c), rank_score(trump, c, &in_suit)))
                    .unwrap_or_default()
            };
        }
        let low: Vec<Vec<Card>> = ms
            .iter()
            .cloned()
            .map(|m| complete(m, &in_suit, Fill::Low))
            .collect();
        let worth_taking = trick_points > 0 || last || rng.gen_bool(0.5);
        if worth_taking {
            // Take it as cheaply as possible; when last, grab points too.
            let mut winners: Vec<Vec<Card>> = low
                .iter()
                .filter(|c| would_win(p, id, c))
                .cloned()
                .collect();
            if last {
                winners.extend(
                    ms.iter()
                        .cloned()
                        .map(|m| complete(m, &in_suit, Fill::Points))
                        .filter(|c| would_win(p, id, c)),
                );
            }
            let best = if last {
                winners
                    .into_iter()
                    .max_by_key(|c| (total_points(c), usize::MAX - rank_score(trump, c, &in_suit)))
            } else {
                winners
                    .into_iter()
                    .min_by_key(|c| rank_score(trump, c, &in_suit))
            };
            if let Some(c) = best {
                return c;
            }
        }
        // Can't (or needn't) win: play the cheapest cards, keeping points back.
        return low
            .into_iter()
            .min_by_key(|c| (total_points(c), rank_score(trump, c, &in_suit)))
            .unwrap_or_default();
    }

    if in_suit.is_empty() && suit != EffectiveSuit::Trump && !partner_winning {
        let trumps = in_suit_of(&hand, EffectiveSuit::Trump);
        let stakes = trick_points >= 10 || (last && trick_points > 0);
        if stakes && trumps.len() >= size {
            let ruff = matchings(tf, &trumps, policy, true, 8)
                .into_iter()
                .filter(|r| r.len() == size && would_win(p, id, r))
                .min_by_key(|r| rank_score(trump, r, &trumps));
            if let Some(r) = ruff {
                return r;
            }
        }
    }

    let fill = if feed { Fill::Points } else { Fill::Low };
    let mut c = in_suit.clone();
    c.extend(pick(trump, &rest_of_hand, size - in_suit.len(), fill));
    c
}

/// Lead: something nobody can beat if possible, otherwise a low card from a short suit.
fn lead(
    p: &PlayPhase,
    hand: &[Card],
    opponents: &[Vec<Card>],
    others: &[Vec<Card>],
    rng: &mut StdRng,
) -> Vec<Card> {
    let trump = p.trump;
    let suits = by_suit(trump, hand);
    let count_in = |cards: &[Card], s: EffectiveSuit| {
        cards
            .iter()
            .filter(|c| trump.effective_suit(**c) == s)
            .count()
    };
    let beats = |x: Card, c: Card| {
        trump.effective_suit(x) == trump.effective_suit(c)
            && trump.compare(x, c) == std::cmp::Ordering::Greater
    };
    // Nobody else holds a higher card of the suit, and no opponent can trump it.
    let safe_from_ruff = |s: EffectiveSuit, n: usize| {
        s == EffectiveSuit::Trump
            || opponents
                .iter()
                .all(|o| count_in(o, s) >= n || count_in(o, EffectiveSuit::Trump) < n)
    };
    let boss_single = |c: Card| {
        others.iter().all(|o| o.iter().all(|x| !beats(*x, c)))
            && safe_from_ruff(trump.effective_suit(c), 1)
    };
    let boss_unit = |u: &[Card]| {
        let top = u
            .iter()
            .copied()
            .max_by(|a, b| trump.compare(*a, *b))
            .unwrap();
        let s = trump.effective_suit(top);
        // No other player holds a higher pair in the suit.
        others.iter().all(|o| {
            let higher: Vec<Card> = o.iter().copied().filter(|x| beats(*x, top)).collect();
            !higher
                .iter()
                .enumerate()
                .any(|(i, x)| higher[i + 1..].contains(x))
        }) && safe_from_ruff(s, u.len())
    };

    // Multi-card units (pairs, tractors) that can't be beaten, biggest and richest first.
    let mut best_unit: Option<Vec<Card>> = None;
    for (s, cards) in &suits {
        for u in multi_units(trump, p.propagated.tractor_requirements, cards) {
            if boss_unit(&u)
                && (*s != EffectiveSuit::Trump || rng.gen_bool(0.3))
                && best_unit
                    .as_ref()
                    .map(|b| (u.len(), total_points(&u)) > (b.len(), total_points(b)))
                    .unwrap_or(true)
            {
                best_unit = Some(u);
            }
        }
    }
    if let Some(u) = best_unit {
        return u;
    }
    // Unbeatable singles in side suits, richest first.
    let boss = hand
        .iter()
        .copied()
        .filter(|c| trump.effective_suit(*c) != EffectiveSuit::Trump && boss_single(*c))
        .max_by_key(|c| points(*c));
    if let Some(c) = boss {
        return vec![c];
    }
    // Otherwise lead low from the shortest side suit, working toward a void.
    let side = suits
        .iter()
        .filter(|(s, v)| **s != EffectiveSuit::Trump && !v.is_empty())
        .min_by_key(|(_, v)| v.len());
    if let Some((_, cards)) = side {
        return pick(trump, cards, 1, Fill::Low);
    }
    // Only trump left: lead a pair if we have one, else the lowest trump.
    let trumps = suits
        .get(&EffectiveSuit::Trump)
        .cloned()
        .unwrap_or_default();
    let units = multi_units(trump, p.propagated.tractor_requirements, &trumps);
    if let Some(u) = units
        .into_iter()
        .min_by_key(|u| rank_score(trump, u, &trumps))
    {
        return u;
    }
    pick(trump, &trumps, 1, Fill::Low)
}
