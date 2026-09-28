//! After-hand review: re-analyze each play a person made, from what they could see at the time,
//! and pick out the moments that changed their side's chances the most.

use std::time::Duration;

use rand::rngs::StdRng;
use rand::SeedableRng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shengji_mechanics::types::{Card, EffectiveSuit, PlayerID};

use super::heuristics::{card_name, counts, hand_cards, points, sort_by_rank, total_points};
use super::play::analyze_with;
use super::{BotConfig, Difficulty, Memory};
use crate::game_state::play_phase::PlayPhase;
use crate::game_state::GameState;

/// One play a person made, with the state as they saw it just before.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedPlay {
    pub view: GameState,
    pub cards: Vec<Card>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Moment {
    /// 1-based trick number.
    pub trick: usize,
    /// True for a strong play, false for a costly one.
    pub good: bool,
    pub played: Vec<Card>,
    pub best: Vec<Card>,
    /// Chance your side won the hand in simulation after your play, and after the best play.
    pub played_win: f64,
    pub best_win: f64,
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HandReview {
    pub moments: Vec<Moment>,
    /// Plays that had a real choice.
    pub reviewed: usize,
    /// Of those, plays that matched the coach (or came within 2%).
    pub matched: usize,
}

fn describe(cards: &[Card]) -> String {
    cards
        .iter()
        .map(|c| card_name(*c))
        .collect::<Vec<_>>()
        .join(" ")
}

fn pct(x: f64) -> String {
    format!("{:.0}%", x * 100.0)
}

fn same_side(p: &PlayPhase, a: PlayerID, b: PlayerID) -> bool {
    p.landlords_team.contains(&a) == p.landlords_team.contains(&b)
}

/// A short, concrete reason a play cost something, when one of the common patterns applies.
fn reason(
    p: &PlayPhase,
    me: PlayerID,
    played: &[Card],
    best: &[Card],
    name: &dyn Fn(PlayerID) -> String,
) -> String {
    let trump = p.trump;
    let winner = p.trick.current_winner();
    let led_trump = p
        .trick
        .trick_format()
        .map(|tf| tf.suit() == EffectiveSuit::Trump)
        .unwrap_or(false);
    let played_trump = played
        .iter()
        .filter(|c| trump.effective_suit(**c) == EffectiveSuit::Trump)
        .count();
    let best_trump = best
        .iter()
        .filter(|c| trump.effective_suit(**c) == EffectiveSuit::Trump)
        .count();
    if let Some(w) = winner {
        if w != me && same_side(p, w, me) {
            if !led_trump && played_trump > best_trump {
                return format!(
                    "{} was already winning the trick, so your trump was spent for nothing.",
                    name(w)
                );
            }
            if total_points(best) > total_points(played) {
                return format!(
                    "{} was winning, so this was the moment to give your side points.",
                    name(w)
                );
            }
        }
        if !same_side(p, w, me) && total_points(played) > total_points(best) {
            let pts = total_points(played) - total_points(best);
            return format!("It handed {pts} extra points to the other side.");
        }
    }
    // Splitting a pair you held, when the better play kept it.
    let hand = counts(&hand_cards(&p.hands, me));
    let played_counts = counts(played);
    let best_counts = counts(best);
    for (card, n) in &played_counts {
        let held = hand.get(card).copied().unwrap_or(0);
        if held >= 2 && *n < held && best_counts.get(card).copied().unwrap_or(0) != 1 {
            return format!("It split your pair of {}.", card_name(*card));
        }
    }
    if points_left_in_hand(p, me) > 0
        && total_points(played) > total_points(best)
        && winner.is_none()
    {
        return "Leading point cards early let the other side win them.".to_string();
    }
    String::new()
}

fn points_left_in_hand(p: &PlayPhase, me: PlayerID) -> usize {
    hand_cards(&p.hands, me).iter().map(|c| points(*c)).sum()
}

/// The analysis of one play: how it compared with the coach's best play.
#[derive(Debug, Clone)]
pub struct PlayVerdict {
    trick: usize,
    chosen: Vec<Card>,
    best: Vec<Card>,
    played_win: f64,
    best_win: f64,
    second: Option<(Vec<Card>, f64)>,
    chosen_is_best: bool,
    /// Standard error of the difference between the two win rates.
    noise: f64,
    reason: String,
}

fn review_config(budget: Duration) -> BotConfig {
    let mut config = BotConfig::for_difficulty(Difficulty::Hard);
    config.samples = 96;
    config.time_budget = budget;
    config
}

/// Analyze one recorded play within `budget`. `None` when there was no real choice.
pub fn review_play(rec: &RecordedPlay, name: &str, budget: Duration) -> Option<PlayVerdict> {
    let mut rng = StdRng::from_entropy();
    let p = match &rec.view {
        GameState::Play(p) => p,
        _ => return None,
    };
    let me = rec.view.player_id(name).ok()?;
    let mut chosen = rec.cards.clone();
    sort_by_rank(p.trump, &mut chosen);
    let config = review_config(budget);
    let a = analyze_with(p, me, &Memory::default(), &config, &mut rng, Some(&chosen));
    if a.candidates.len() < 2 {
        return None;
    }
    let ranked = a.ranked();
    let best = *ranked.first()?;
    let chosen_idx = a.candidates.iter().position(|c| *c == chosen)?;
    let n = a.stats[chosen_idx].n.min(a.stats[best].n);
    if n == 0 {
        return None;
    }
    let win = |i: usize| {
        let s = a.stats[i];
        if s.n == 0 {
            0.0
        } else {
            s.wins / s.n as f64
        }
    };
    let (played_win, best_win) = (win(chosen_idx), win(best));
    let var = |w: f64| w * (1.0 - w) / n as f64;
    let noise = (var(played_win) + var(best_win)).sqrt().max(0.02);
    let players = p.propagated.players.clone();
    let who = move |id: PlayerID| -> String {
        if id == me {
            "You".to_string()
        } else {
            players
                .iter()
                .find(|pl| pl.id == id)
                .map(|pl| pl.name.split(" (").next().unwrap_or(&pl.name).to_string())
                .unwrap_or_default()
        }
    };
    let best_cards = a.candidates[best].clone();
    let reason = if best != chosen_idx {
        reason(p, me, &chosen, &best_cards, &who)
    } else {
        String::new()
    };
    let second = if best == chosen_idx {
        ranked.get(1).map(|&i| (a.candidates[i].clone(), win(i)))
    } else {
        None
    };
    Some(PlayVerdict {
        trick: p.played_tricks.len() + 1,
        chosen,
        best: best_cards,
        played_win,
        best_win,
        second,
        chosen_is_best: best == chosen_idx,
        noise,
        reason,
    })
}

/// Pick the key moments from analyzed plays: the costliest plays and the best finds, keeping
/// only differences clearly larger than the simulation's noise.
pub fn summarize(verdicts: &[PlayVerdict]) -> HandReview {
    let mut costly: Vec<(f64, Moment)> = vec![];
    let mut good: Vec<(f64, Moment)> = vec![];
    let mut matched = 0;
    for v in verdicts {
        let gap = v.best_win - v.played_win;
        if v.chosen_is_best || gap < 0.05 {
            matched += 1;
        }
        let clear = |d: f64| d >= 0.08 && d >= 1.5 * v.noise;
        if !v.chosen_is_best && clear(gap) {
            let text = format!(
                "You played {}. With {} your side won {} of simulated hands, against {} after your play.{}",
                describe(&v.chosen),
                describe(&v.best),
                pct(v.best_win),
                pct(v.played_win),
                if v.reason.is_empty() { String::new() } else { format!(" {}", v.reason) }
            );
            costly.push((
                gap,
                Moment {
                    trick: v.trick,
                    good: false,
                    played: v.chosen.clone(),
                    best: v.best.clone(),
                    played_win: v.played_win,
                    best_win: v.best_win,
                    title: format!("Trick {} · costly", v.trick),
                    text,
                },
            ));
        } else if v.chosen_is_best {
            if let Some((second, second_win)) = &v.second {
                let margin = v.best_win - second_win;
                if clear(margin) {
                    let text = format!(
                        "You found the best play, {} ({}). The next best, {}, won {}.",
                        describe(&v.chosen),
                        pct(v.best_win),
                        describe(second),
                        pct(*second_win)
                    );
                    good.push((
                        margin,
                        Moment {
                            trick: v.trick,
                            good: true,
                            played: v.chosen.clone(),
                            best: v.chosen.clone(),
                            played_win: v.played_win,
                            best_win: v.best_win,
                            title: format!("Trick {} · great play", v.trick),
                            text,
                        },
                    ));
                }
            }
        }
    }
    let by_size = |a: &(f64, Moment), b: &(f64, Moment)| {
        b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
    };
    costly.sort_by(by_size);
    good.sort_by(by_size);
    let mut moments: Vec<Moment> = costly
        .into_iter()
        .take(3)
        .chain(good.into_iter().take(2))
        .map(|(_, m)| m)
        .collect();
    moments.sort_by_key(|m| m.trick);
    HandReview {
        moments,
        reviewed: verdicts.len(),
        matched,
    }
}

/// Review every recorded play for `name` in one go (native callers with time to spare).
pub fn review_hand(plays: &[RecordedPlay], name: &str) -> HandReview {
    let verdicts: Vec<PlayVerdict> = plays
        .iter()
        .filter_map(|rec| review_play(rec, name, Duration::from_millis(1500)))
        .collect();
    summarize(&verdicts)
}
