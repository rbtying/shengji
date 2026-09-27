//! A computer player.
//!
//! The bot only ever looks at the state the server sends to its own seat (other hands and the
//! kitty are redacted). Card play uses perfect-information Monte Carlo (PIMC): the hidden cards are
//! dealt out many plausible ways, each candidate play is played out to the end of the hand with a
//! fast heuristic policy, and the candidate with the best average result is chosen. Bidding,
//! burying the kitty and calling friends use simple heuristics.

use std::collections::HashMap;
use std::time::Duration;

use rand::rngs::StdRng;
use rand::SeedableRng;

use shengji_mechanics::trick::Trick;
use shengji_mechanics::types::{Card, EffectiveSuit, PlayerID, Trump};

use crate::game_state::GameState;
use crate::interactive::Action;

mod heuristics;
mod play;
mod setup;

#[derive(Debug, Clone)]
pub struct BotConfig {
    /// Maximum number of sampled deals per card-play decision.
    pub samples: usize,
    /// Wall-clock budget per card-play decision.
    pub time_budget: Duration,
    /// Worker threads used for simulation.
    pub threads: usize,
    /// Whether the bot finishes completed tricks (so play keeps moving).
    pub finish_tricks: bool,
    /// Whether the bot starts the next hand once a hand is over.
    pub start_new_games: bool,
    /// Skip simulation and play by the fast heuristic alone (a weaker baseline for testing).
    pub heuristic_only: bool,
}

impl Default for BotConfig {
    fn default() -> Self {
        BotConfig {
            samples: 64,
            time_budget: Duration::from_millis(1500),
            threads: std::thread::available_parallelism()
                .map(|n| n.get().min(8))
                .unwrap_or(2),
            finish_tricks: true,
            start_new_games: false,
            heuristic_only: false,
        }
    }
}

/// An action the bot wants to take, and how long it should wait before taking it (so humans can
/// follow along, and so other players get a chance to act first).
#[derive(Debug, Clone)]
pub struct Decision {
    pub action: Action,
    pub delay: Duration,
    pub note: String,
}

impl Decision {
    fn new(action: Action, delay_ms: u64, note: impl Into<String>) -> Self {
        Decision {
            action,
            delay: Duration::from_millis(delay_ms),
            note: note.into(),
        }
    }
}

/// What the bot remembers between state updates: the tricks completed so far this hand. The
/// server state only carries the current and previous trick.
#[derive(Debug, Default, Clone)]
pub(crate) struct Memory {
    pub(crate) tricks: Vec<Trick>,
    last_trick_key: Option<String>,
    hand_marker: Option<(PlayerID, Trump, usize)>,
}

impl Memory {
    fn observe(&mut self, state: &GameState) {
        let p = match state {
            GameState::Play(p) => p,
            _ => {
                self.tricks.clear();
                self.last_trick_key = None;
                self.hand_marker = None;
                return;
            }
        };
        let marker = (p.landlord, p.trump, p.propagated.num_games_finished);
        if self.hand_marker != Some(marker) {
            self.tricks.clear();
            self.last_trick_key = None;
            self.hand_marker = Some(marker);
        }
        if let Some(t) = &p.last_trick {
            let key = serde_json::to_string(t).unwrap_or_default();
            if self.last_trick_key.as_deref() != Some(key.as_str()) {
                self.tricks.push(t.clone());
                self.last_trick_key = Some(key);
            }
        }
    }

    /// Suits each player has shown out of, from failures to follow suit.
    pub(crate) fn voids(&self, current: &Trick) -> HashMap<PlayerID, Vec<EffectiveSuit>> {
        let mut voids: HashMap<PlayerID, Vec<EffectiveSuit>> = HashMap::new();
        for trick in self.tricks.iter().chain(std::iter::once(current)) {
            let tf = match trick.trick_format() {
                Some(tf) => tf,
                None => continue,
            };
            if tf.is_rainbow() {
                continue;
            }
            let trump = trick.trump();
            for pc in trick.played_cards().iter().skip(1) {
                let in_suit = pc
                    .cards
                    .iter()
                    .filter(|c| **c != Card::Unknown && trump.effective_suit(**c) == tf.suit())
                    .count();
                if in_suit < tf.size() {
                    let v = voids.entry(pc.id).or_default();
                    if !v.contains(&tf.suit()) {
                        v.push(tf.suit());
                    }
                }
            }
        }
        voids
    }

    /// Every card played so far this hand, as far as we saw.
    pub(crate) fn played_cards(&self, current: &Trick) -> Vec<Card> {
        self.tricks
            .iter()
            .chain(std::iter::once(current))
            .flat_map(|t| t.played_cards().iter())
            .flat_map(|pc| pc.cards.iter().copied())
            .filter(|c| *c != Card::Unknown)
            .collect()
    }
}

pub struct Bot {
    name: String,
    config: BotConfig,
    memory: Memory,
    rng: StdRng,
}

impl Bot {
    pub fn new(name: impl Into<String>, config: BotConfig) -> Self {
        Bot {
            name: name.into(),
            config,
            memory: Memory::default(),
            rng: StdRng::from_entropy(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Record a state update. Call this for every state received, including ones that don't need
    /// an action, so trick history stays complete.
    pub fn observe(&mut self, state: &GameState) {
        self.memory.observe(state);
    }

    /// Decide what to do next given the latest state, if anything.
    pub fn decide(&mut self, state: &GameState) -> Option<Decision> {
        let me = state.player_id(&self.name).ok()?;
        if !state.is_player(me) {
            return None;
        }
        match state {
            GameState::Initialize(_) => None,
            GameState::Draw(d) => setup::draw_decision(d, me, &mut self.rng),
            GameState::Exchange(e) => setup::exchange_decision(e, me),
            GameState::Play(p) => {
                if p.game_finished() {
                    if self.config.start_new_games && p.landlord == me {
                        return Some(Decision::new(Action::StartNewGame, 4000, "next hand"));
                    }
                    return None;
                }
                let trick = &p.trick;
                if trick.next_player().is_none() && !trick.played_cards().is_empty() {
                    if self.config.finish_tricks {
                        return Some(Decision::new(Action::EndTrick, 1800, "finish trick"));
                    }
                    return None;
                }
                if trick.next_player() == Some(me) {
                    let (cards, note) =
                        play::choose_play(p, me, &self.memory, &self.config, &mut self.rng);
                    return Some(Decision::new(Action::PlayCards(cards), 300, note));
                }
                None
            }
        }
    }
}
