//! Card play: candidate generation, determinization and PIMC rollouts.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use web_time::Instant;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use shengji_mechanics::player::Player;
use shengji_mechanics::trick::Trick;
use shengji_mechanics::types::{Card, EffectiveSuit, PlayerID, Rank};

use super::heuristics::{
    by_suit, card_name, hand_cards, matchings, multi_units, pick, points, rank_score, sort_by_rank,
    subtract, total_points, Fill,
};
use super::{played_cards, voids, BotConfig, Hint, HintOption, Memory};
use crate::game_state::play_phase::PlayPhase;
use crate::settings::GameMode;

/// Who we believe is on the landlord's side while simulating.
pub(crate) struct Sides {
    landlord_side: Vec<PlayerID>,
}

impl Sides {
    /// In a fully-dealt world, unrevealed friends are whoever holds the called cards.
    pub(crate) fn of(p: &PlayPhase) -> Self {
        let mut landlord_side = p.landlords_team.clone();
        if let GameMode::FindingFriends { friends, .. } = &p.game_mode {
            for f in friends.iter().filter(|f| f.player_id.is_none()) {
                for pl in &p.propagated.players {
                    let holds = p
                        .hands
                        .get(pl.id)
                        .map(|h| h.get(&f.card).copied().unwrap_or(0) > 0)
                        .unwrap_or(false);
                    if holds && !landlord_side.contains(&pl.id) {
                        landlord_side.push(pl.id);
                    }
                }
            }
        }
        Sides { landlord_side }
    }

    fn same(&self, a: PlayerID, b: PlayerID) -> bool {
        self.landlord_side.contains(&a) == self.landlord_side.contains(&b)
    }
}

/// How one candidate fared across the simulated deals.
#[derive(Clone, Copy, Default)]
pub(crate) struct Stats {
    value: f64,
    pub(crate) wins: f64,
    defender_points: f64,
    pub(crate) n: usize,
}

impl Stats {
    fn add(&mut self, o: &Outcome) {
        self.value += o.value;
        self.wins += if o.won { 1.0 } else { 0.0 };
        self.defender_points += o.defender_points;
        self.n += 1;
    }

    fn merge(&mut self, o: &Stats) {
        self.value += o.value;
        self.wins += o.wins;
        self.defender_points += o.defender_points;
        self.n += o.n;
    }

    fn mean(&self) -> f64 {
        if self.n == 0 {
            f64::NEG_INFINITY
        } else {
            self.value / self.n as f64
        }
    }
}

pub(crate) struct Analysis {
    pub(crate) candidates: Vec<Vec<Card>>,
    pub(crate) stats: Vec<Stats>,
    pub(crate) note: String,
}

impl Analysis {
    /// Candidate indices, best first; only those that were simulated at least once.
    pub(crate) fn ranked(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.candidates.len())
            .filter(|i| self.stats[*i].n > 0)
            .collect();
        idx.sort_by(|a, b| {
            self.stats[*b]
                .mean()
                .partial_cmp(&self.stats[*a].mean())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        idx
    }
}

/// Simulate every candidate play for `me` over sampled deals of the hidden cards.
pub(crate) fn analyze(
    p: &PlayPhase,
    me: PlayerID,
    memory: &Memory,
    config: &BotConfig,
    rng: &mut StdRng,
) -> Analysis {
    analyze_with(p, me, memory, config, rng, None)
}

/// Like `analyze`, also simulating `extra` (e.g. the play a person actually made) if it isn't
/// already a candidate.
pub(crate) fn analyze_with(
    p: &PlayPhase,
    me: PlayerID,
    memory: &Memory,
    config: &BotConfig,
    rng: &mut StdRng,
    extra: Option<&[Card]>,
) -> Analysis {
    let history = memory.history(p);
    let mut cands = candidates_with_throws(p, me, history);
    if let Some(extra) = extra {
        let mut e = extra.to_vec();
        sort_by_rank(p.trump, &mut e);
        if !cands.contains(&e) {
            cands.push(e);
        }
    }
    let n = cands.len();
    let start = Instant::now();
    let base_levels = levels(&p.propagated.players);
    let totals = Mutex::new(vec![Stats::default(); n]);
    let samples_started = AtomicUsize::new(0);
    let no_worlds = AtomicBool::new(false);
    let seeds: Vec<u64> = (0..config.threads.max(1)).map(|_| rng.gen()).collect();

    let worker = |seed: u64| {
        let mut trng = StdRng::seed_from_u64(seed);
        let mut local = vec![Stats::default(); n];
        loop {
            if start.elapsed() > config.time_budget
                || samples_started.fetch_add(1, Ordering::Relaxed) >= config.samples
            {
                break;
            }
            let world = match determinize(p, me, history, &mut trng) {
                Some(w) => w,
                None => {
                    no_worlds.store(true, Ordering::Relaxed);
                    break;
                }
            };
            let sides = Sides::of(&world);
            for (i, c) in cands.iter().enumerate() {
                let mut sim = world.clone();
                if sim.play_cards(me, c).is_err() {
                    continue;
                }
                if let Some(o) = rollout(sim, me, &base_levels, &sides, &mut trng) {
                    local[i].add(&o);
                }
            }
        }
        let mut t = totals.lock().unwrap();
        for (i, s) in local.iter().enumerate() {
            t[i].merge(s);
        }
    };

    if n > 1 {
        if seeds.len() == 1 {
            // Single-threaded platforms (WebAssembly) run the simulation inline.
            worker(seeds[0]);
        } else {
            std::thread::scope(|scope| {
                for seed in &seeds {
                    let worker = &worker;
                    scope.spawn(move || worker(*seed));
                }
            });
        }
    }

    let stats = totals.into_inner().unwrap();
    let simulated = stats.iter().map(|s| s.n).max().unwrap_or(0);
    let note = if n <= 1 {
        "forced".to_string()
    } else if simulated == 0 {
        if no_worlds.load(Ordering::Relaxed) {
            "could not sample hidden hands".to_string()
        } else {
            "no simulations finished".to_string()
        }
    } else {
        format!(
            "{n} candidates, {simulated} samples, {} ms",
            start.elapsed().as_millis()
        )
    };
    Analysis {
        candidates: cands,
        stats,
        note,
    }
}

pub(crate) fn choose_play(
    p: &PlayPhase,
    me: PlayerID,
    memory: &Memory,
    config: &BotConfig,
    rng: &mut StdRng,
) -> (Vec<Card>, String) {
    if config.mistake_rate > 0.0 && rng.gen_bool(config.mistake_rate.min(1.0)) {
        let cands = candidates(p, me);
        if let Some(c) = cands.choose(rng) {
            return (c.clone(), "careless play".into());
        }
    }
    if config.heuristic_only {
        let sides = Sides::of(p);
        return (policy_play(p, me, &sides, rng), "heuristic".into());
    }
    let a = analyze(p, me, memory, config, rng);
    if a.candidates.len() == 1 {
        return (a.candidates[0].clone(), a.note);
    }
    match a.ranked().first() {
        Some(i) => (
            a.candidates[*i].clone(),
            format!("{}, value {:.3}", a.note, a.stats[*i].mean()),
        ),
        None => (
            policy_play(p, me, &Sides::of(p), rng),
            format!("{}; heuristic", a.note),
        ),
    }
}

fn describe(cards: &[Card]) -> String {
    cards
        .iter()
        .map(|c| card_name(*c))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn play_hint(
    p: &PlayPhase,
    me: PlayerID,
    memory: &Memory,
    config: &BotConfig,
    rng: &mut StdRng,
) -> Option<Hint> {
    let a = analyze(p, me, memory, config, rng);
    if a.candidates.len() == 1 {
        return Some(Hint {
            cards: a.candidates[0].clone(),
            summary: format!(
                "{} is your only sensible play here.",
                describe(&a.candidates[0])
            ),
            options: vec![],
        });
    }
    let ranked = a.ranked();
    let best = *ranked.first()?;
    let options: Vec<HintOption> = ranked
        .iter()
        .take(4)
        .map(|i| {
            let s = a.stats[*i];
            HintOption {
                cards: a.candidates[*i].clone(),
                win_rate: s.wins / s.n as f64,
                defender_points: s.defender_points / s.n as f64,
                samples: s.n,
            }
        })
        .collect();
    let b = &options[0];
    let mut summary = format!(
        "Play {}. Your side won the hand in {:.0}% of {} simulated deals.",
        describe(&a.candidates[best]),
        b.win_rate * 100.0,
        b.samples
    );
    if let Some(next) = options.get(1) {
        summary.push_str(&format!(
            " Next best: {} ({:.0}%).",
            describe(&next.cards),
            next.win_rate * 100.0
        ));
    }
    Some(Hint {
        cards: a.candidates[best].clone(),
        summary,
        options,
    })
}

fn level_value(p: &Player) -> i64 {
    let l = match p.level {
        Rank::Number(n) => n.as_u32() as i64,
        Rank::NoTrump => 15,
    };
    p.metalevel as i64 * 20 + l
}

fn levels(players: &[Player]) -> HashMap<PlayerID, i64> {
    players.iter().map(|p| (p.id, level_value(p))).collect()
}

pub(crate) struct Outcome {
    value: f64,
    won: bool,
    defender_points: f64,
}

/// Outcome of a finished hand from `me`'s point of view: how far I advanced relative to the
/// players against me, plus a small term for the point margin so close outcomes still differ.
fn evaluate(s: &PlayPhase, me: PlayerID, base: &HashMap<PlayerID, i64>) -> Outcome {
    let (defender_points, _) = s.calculate_points();
    let deck_points: usize = s.decks.iter().map(|d| d.points()).sum();
    let mine_is_landlord_side = s.landlords_team.contains(&me);
    let margin = defender_points as f64 / deck_points.max(1) as f64;
    let point_term = if mine_is_landlord_side {
        -margin
    } else {
        margin
    };

    let (level_term, won) = match s.finish_game() {
        Ok((init, landlord_won, _)) => {
            let after = levels(&init.players);
            let delta = |id: PlayerID| {
                after.get(&id).copied().unwrap_or(0) - base.get(&id).copied().unwrap_or(0)
            };
            let opponents: Vec<PlayerID> = s
                .propagated
                .players
                .iter()
                .map(|p| p.id)
                .filter(|id| s.landlords_team.contains(id) != mine_is_landlord_side)
                .collect();
            let opp = if opponents.is_empty() {
                0.0
            } else {
                opponents.iter().map(|id| delta(*id) as f64).sum::<f64>() / opponents.len() as f64
            };
            (
                delta(me) as f64 - opp,
                landlord_won == mine_is_landlord_side,
            )
        }
        Err(_) => (0.0, false),
    };
    Outcome {
        value: level_term + 0.5 * point_term,
        won,
        defender_points: defender_points as f64,
    }
}

fn rollout(
    mut s: PlayPhase,
    me: PlayerID,
    base: &HashMap<PlayerID, i64>,
    sides: &Sides,
    rng: &mut StdRng,
) -> Option<Outcome> {
    for _ in 0..5000 {
        if s.game_finished() {
            return Some(evaluate(&s, me, base));
        }
        let next = match s.trick.next_player() {
            Some(n) => n,
            None => {
                s.finish_trick().ok()?;
                continue;
            }
        };
        let cards = policy_play(&s, next, sides, rng);
        if s.play_cards(next, &cards).is_err() {
            let fallback = candidates(&s, next);
            let c = fallback.first()?;
            s.play_cards(next, c).ok()?;
        }
    }
    None
}

/// Candidates plus, when leading, throws of cards that card-counting says nobody can beat.
pub(crate) fn candidates_with_throws(
    p: &PlayPhase,
    me: PlayerID,
    history: &[Trick],
) -> Vec<Vec<Card>> {
    let mut out = candidates(p, me);
    if p.trick.trick_format().is_none() {
        for t in safe_throws(p, me, history) {
            if !out.contains(&t) {
                out.push(t);
            }
        }
    }
    out
}

/// For each side suit, the units we hold that no unseen cards can beat, combined into one
/// throw when there are at least two of them.
fn safe_throws(p: &PlayPhase, me: PlayerID, history: &[Trick]) -> Vec<Vec<Card>> {
    let trump = p.trump;
    let hand = hand_cards(&p.hands, me);
    let mut full: Vec<Card> = p.decks.iter().flat_map(|d| d.cards()).collect();
    full = subtract(&full, &p.removed_cards);
    let mut seen = hand.clone();
    seen.extend(played_cards(history, &p.trick));
    seen.extend(p.kitty.iter().copied().filter(|c| *c != Card::Unknown));
    let unseen = subtract(&full, &seen);

    let mut throws = vec![];
    for (suit, cards) in by_suit(trump, &hand) {
        if suit == EffectiveSuit::Trump {
            continue;
        }
        let unseen_in_suit: Vec<Card> = unseen
            .iter()
            .copied()
            .filter(|c| trump.effective_suit(*c) == suit)
            .collect();
        let unseen_counts = super::heuristics::counts(&unseen_in_suit);
        // A unit of width w is safe if no unseen card above its lowest card has w copies left
        // unseen (for singles: no higher card is unseen at all). Tractors are treated like
        // their width, which is conservative.
        let beats = |lowest: Card, width: usize| {
            unseen_counts.iter().any(|(c, n)| {
                *n >= width && trump.compare(*c, lowest) == std::cmp::Ordering::Greater
            })
        };
        let mut units: Vec<Vec<Card>> =
            multi_units(trump, p.propagated.tractor_requirements, &cards);
        units.sort_by_key(|u| std::cmp::Reverse(u.len()));
        let mut used: Vec<Card> = vec![];
        let mut throw: Vec<Card> = vec![];
        let mut pieces = 0;
        for u in units {
            let remaining = subtract(&cards, &used);
            if subtract(&u, &remaining).is_empty() {
                let width = super::heuristics::counts(&u)
                    .values()
                    .copied()
                    .min()
                    .unwrap_or(1);
                if !beats(u[0], width) {
                    used.extend(u.iter().copied());
                    throw.extend(u);
                    pieces += 1;
                }
            }
        }
        for c in subtract(&cards, &used).into_iter().rev() {
            if !beats(c, 1) {
                throw.push(c);
                pieces += 1;
            } else {
                break;
            }
        }
        if pieces >= 2 {
            sort_by_rank(trump, &mut throw);
            throws.push(throw);
        }
    }
    throws
}

/// Deal the cards we can't see to the other players (and the kitty, if we didn't bury it),
/// respecting hand sizes and the suits each player has shown out of.
pub(crate) fn determinize(
    p: &PlayPhase,
    me: PlayerID,
    history: &[Trick],
    rng: &mut StdRng,
) -> Option<PlayPhase> {
    let mut full: Vec<Card> = p.decks.iter().flat_map(|d| d.cards()).collect();
    full = subtract(&full, &p.removed_cards);
    let mut known = hand_cards(&p.hands, me);
    known.extend(played_cards(history, &p.trick));
    known.extend(p.kitty.iter().copied().filter(|c| *c != Card::Unknown));
    let mut pool = subtract(&full, &known);

    #[derive(Clone, Copy, PartialEq)]
    enum Holder {
        Player(PlayerID),
        Kitty,
    }
    let mut needs: Vec<(Holder, usize)> = vec![];
    for pl in &p.propagated.players {
        if pl.id == me {
            continue;
        }
        let n = p
            .hands
            .get(pl.id)
            .ok()
            .and_then(|h| h.get(&Card::Unknown).copied())
            .unwrap_or(0);
        if n > 0 {
            needs.push((Holder::Player(pl.id), n));
        }
    }
    let kitty_unknown = p.kitty.iter().filter(|c| **c == Card::Unknown).count();
    if kitty_unknown > 0 {
        needs.push((Holder::Kitty, kitty_unknown));
    }
    let total: usize = needs.iter().map(|(_, n)| n).sum();
    if pool.len() < total {
        return None;
    }
    pool.shuffle(rng);
    // If we missed some history the pool is too big; drop the excess at random.
    pool.truncate(total);

    let voids = voids(history, &p.trick);
    let trump = p.trump;
    let eligible = |h: Holder, c: Card| match h {
        Holder::Kitty => true,
        Holder::Player(id) => voids
            .get(&id)
            .map(|v| !v.contains(&trump.effective_suit(c)))
            .unwrap_or(true),
    };

    let mut assignment: Option<Vec<Vec<Card>>> = None;
    for attempt in 0..40 {
        let respect_voids = attempt < 39;
        let mut cap: Vec<usize> = needs.iter().map(|(_, n)| *n).collect();
        let mut dealt: Vec<Vec<Card>> = vec![vec![]; needs.len()];
        let mut cards = pool.clone();
        cards.shuffle(rng);
        if respect_voids {
            cards.sort_by_key(|c| needs.iter().filter(|(h, _)| eligible(*h, *c)).count());
        }
        let mut ok = true;
        for c in cards {
            let options: Vec<usize> = (0..needs.len())
                .filter(|i| cap[*i] > 0 && (!respect_voids || eligible(needs[*i].0, c)))
                .collect();
            let weight: usize = options.iter().map(|i| cap[*i]).sum();
            if weight == 0 {
                ok = false;
                break;
            }
            let mut r = rng.gen_range(0..weight);
            let mut chosen = options[0];
            for i in options {
                if r < cap[i] {
                    chosen = i;
                    break;
                }
                r -= cap[i];
            }
            cap[chosen] -= 1;
            dealt[chosen].push(c);
        }
        if ok {
            assignment = Some(dealt);
            break;
        }
    }
    let assignment = assignment?;

    let mut s = p.clone();
    for ((holder, n), cards) in needs.iter().zip(assignment) {
        match holder {
            Holder::Player(id) => {
                s.hands
                    .remove(*id, std::iter::repeat_n(Card::Unknown, *n))
                    .ok()?;
                s.hands.add(*id, cards).ok()?;
            }
            Holder::Kitty => {
                let mut it = cards.into_iter();
                for k in s.kitty.iter_mut() {
                    if *k == Card::Unknown {
                        *k = it.next()?;
                    }
                }
            }
        }
    }
    Some(s)
}

fn dedupe_sorted(p: &PlayPhase, mut v: Vec<Vec<Card>>) -> Vec<Vec<Card>> {
    for c in v.iter_mut() {
        sort_by_rank(p.trump, c);
    }
    let mut out: Vec<Vec<Card>> = vec![];
    for c in v {
        if !c.is_empty() && !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

/// Plays worth considering for `id` right now. Only uses `id`'s own hand and the table.
pub(crate) fn candidates(p: &PlayPhase, id: PlayerID) -> Vec<Vec<Card>> {
    let trump = p.trump;
    let hand = hand_cards(&p.hands, id);
    let policy = p.propagated.trick_draw_policy;
    let mut out: Vec<Vec<Card>> = vec![];

    match p.trick.trick_format() {
        None => {
            for cards in by_suit(trump, &hand).values() {
                out.extend(multi_units(trump, p.propagated.tractor_requirements, cards));
                out.push(vec![*cards.last().unwrap()]);
                out.push(vec![cards[0]]);
                if let Some(pc) = cards.iter().rev().find(|c| points(**c) > 0) {
                    out.push(vec![*pc]);
                }
            }
        }
        Some(tf) => {
            let suit = tf.suit();
            let size = tf.size();
            let (in_suit, others): (Vec<Card>, Vec<Card>) =
                hand.iter().partition(|c| trump.effective_suit(**c) == suit);
            if in_suit.len() >= size {
                let mut ms = matchings(tf, &in_suit, policy, false, 8);
                if ms.is_empty() {
                    ms.push(vec![]);
                }
                for m in ms {
                    let need = size.saturating_sub(m.len());
                    let rest = subtract(&in_suit, &m);
                    for mode in [Fill::Low, Fill::Points, Fill::High] {
                        let mut c = m.clone();
                        c.extend(pick(trump, &rest, need, mode));
                        if c.len() == size {
                            out.push(c);
                        }
                    }
                }
            } else {
                let need = size - in_suit.len();
                for mode in [Fill::Low, Fill::Points] {
                    let mut c = in_suit.clone();
                    c.extend(pick(trump, &others, need, mode));
                    out.push(c);
                }
                if in_suit.is_empty() && suit != EffectiveSuit::Trump {
                    let trumps: Vec<Card> = hand
                        .iter()
                        .copied()
                        .filter(|c| trump.effective_suit(*c) == EffectiveSuit::Trump)
                        .collect();
                    if trumps.len() >= size {
                        let ruffs = matchings(tf, &trumps, policy, true, 16);
                        let mut ruffs: Vec<Vec<Card>> =
                            ruffs.into_iter().filter(|r| r.len() == size).collect();
                        ruffs.sort_by_key(|r| rank_score(trump, r, &trumps));
                        if let Some(r) = ruffs.first() {
                            out.push(r.clone());
                        }
                        if let Some(r) = ruffs.last() {
                            out.push(r.clone());
                        }
                    }
                }
            }
        }
    }

    dedupe_sorted(p, out)
        .into_iter()
        .filter(|c| p.can_play_cards(id, c).is_ok())
        .collect()
}

/// Fast heuristic play used inside rollouts (and as a fallback).
pub(crate) fn policy_play(
    p: &PlayPhase,
    id: PlayerID,
    sides: &Sides,
    rng: &mut StdRng,
) -> Vec<Card> {
    let trump = p.trump;
    let hand = hand_cards(&p.hands, id);
    let policy = p.propagated.trick_draw_policy;

    let tf = match p.trick.trick_format() {
        None => {
            let suits = by_suit(trump, &hand);
            let non_trump: Vec<&Vec<Card>> = suits
                .iter()
                .filter(|(s, _)| **s != EffectiveSuit::Trump)
                .map(|(_, v)| v)
                .collect();
            let cards: &Vec<Card> = if !non_trump.is_empty() && rng.gen_bool(0.8) {
                non_trump.choose(rng).unwrap()
            } else {
                suits.values().next_back().unwrap()
            };
            let units = multi_units(trump, p.propagated.tractor_requirements, cards);
            if !units.is_empty() && rng.gen_bool(0.7) {
                return units
                    .into_iter()
                    .max_by_key(|u| (u.len(), rank_score(trump, u, cards)))
                    .unwrap();
            }
            return if rng.gen_bool(0.6) {
                vec![*cards.last().unwrap()]
            } else {
                vec![cards[0]]
            };
        }
        Some(tf) => tf,
    };

    let suit = tf.suit();
    let size = tf.size();
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
    let (in_suit, others): (Vec<Card>, Vec<Card>) =
        hand.iter().partition(|c| trump.effective_suit(**c) == suit);
    let fill = if partner_winning {
        Fill::Points
    } else {
        Fill::Low
    };

    if in_suit.len() >= size {
        let ms = matchings(tf, &in_suit, policy, false, 6);
        let m = if partner_winning {
            ms.into_iter()
                .min_by_key(|m| rank_score(trump, m, &in_suit))
        } else {
            ms.into_iter()
                .max_by_key(|m| rank_score(trump, m, &in_suit))
        }
        .unwrap_or_default();
        let rest = subtract(&in_suit, &m);
        let mut c = m;
        let need = size.saturating_sub(c.len());
        c.extend(pick(trump, &rest, need, fill));
        return c;
    }

    if in_suit.is_empty()
        && suit != EffectiveSuit::Trump
        && !partner_winning
        && (trick_points >= 10 || rng.gen_bool(0.2))
    {
        let trumps: Vec<Card> = hand
            .iter()
            .copied()
            .filter(|c| trump.effective_suit(*c) == EffectiveSuit::Trump)
            .collect();
        if trumps.len() >= size {
            if let Some(r) = matchings(tf, &trumps, policy, true, 8)
                .into_iter()
                .filter(|r| r.len() == size)
                .min_by_key(|r| rank_score(trump, r, &trumps))
            {
                return r;
            }
        }
    }

    let mut c = in_suit.clone();
    c.extend(pick(trump, &others, size - in_suit.len(), fill));
    c
}
