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
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shengji_mechanics::trick::Trick;
use shengji_mechanics::types::{Card, EffectiveSuit, PlayerID, Trump};

use crate::game_state::play_phase::PlayPhase;
use crate::game_state::GameState;
use crate::interactive::Action;

mod heuristics;
mod play;
mod review;
mod setup;

pub use review::{
    review_hand, review_play, summarize, HandReview, Moment, PlayVerdict, RecordedPlay,
};

/// How strongly a bot plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Default)]
pub enum Difficulty {
    /// Simple rules with frequent mistakes. Good for learning.
    Beginner,
    /// Simple rules, no simulation.
    Easy,
    /// Light simulation.
    Intermediate,
    /// Full simulation.
    #[default]
    Hard,
    /// Deep simulation.
    Expert,
}

impl Difficulty {
    pub const ALL: [Difficulty; 5] = [
        Difficulty::Beginner,
        Difficulty::Easy,
        Difficulty::Intermediate,
        Difficulty::Hard,
        Difficulty::Expert,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Beginner => "beginner",
            Difficulty::Easy => "easy",
            Difficulty::Intermediate => "intermediate",
            Difficulty::Hard => "hard",
            Difficulty::Expert => "expert",
        }
    }
}

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
    /// Skip simulation and play by the fast heuristic alone.
    pub heuristic_only: bool,
    /// Chance of playing a random legal candidate instead of the chosen play.
    pub mistake_rate: f64,
}

impl Default for BotConfig {
    fn default() -> Self {
        BotConfig::for_difficulty(Difficulty::Hard)
    }
}

/// Simulation threads to use: one on WebAssembly (no threads there), else up to eight.
fn default_threads() -> usize {
    if cfg!(target_arch = "wasm32") {
        1
    } else {
        std::thread::available_parallelism()
            .map(|n| n.get().min(8))
            .unwrap_or(2)
    }
}

impl BotConfig {
    pub fn for_difficulty(d: Difficulty) -> Self {
        let (samples, ms, heuristic_only, mistake_rate) = match d {
            Difficulty::Beginner => (0, 0, true, 0.35),
            Difficulty::Easy => (0, 0, true, 0.0),
            Difficulty::Intermediate => (16, 500, false, 0.0),
            Difficulty::Hard => (64, 1500, false, 0.0),
            Difficulty::Expert => (200, 4000, false, 0.0),
        };
        BotConfig {
            samples,
            time_budget: Duration::from_millis(ms),
            threads: default_threads(),
            finish_tricks: true,
            start_new_games: false,
            heuristic_only,
            mistake_rate,
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

/// One option considered for a hint, with how it fared in simulation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HintOption {
    pub cards: Vec<Card>,
    /// Fraction of simulated deals in which your side won the hand.
    pub win_rate: f64,
    /// Average points the defending side ended with.
    pub defender_points: f64,
    pub samples: usize,
}

/// Advice for a human player.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Hint {
    /// The suggested cards (to play, or to bury).
    pub cards: Vec<Card>,
    pub summary: String,
    /// Options compared, best first (card play only).
    pub options: Vec<HintOption>,
}

/// What the bot remembers between state updates: the tricks completed so far this hand. Newer
/// servers record these in the state; with older ones, the bot collects them as it watches.
#[derive(Debug, Default, Clone)]
pub(crate) struct Memory {
    tricks: Vec<Trick>,
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

    /// The best record of this hand's completed tricks.
    pub(crate) fn history<'a>(&'a self, p: &'a PlayPhase) -> &'a [Trick] {
        if p.played_tricks.len() >= self.tricks.len() {
            &p.played_tricks
        } else {
            &self.tricks
        }
    }
}

/// Suits each player has shown out of, from failures to follow suit.
pub(crate) fn voids(history: &[Trick], current: &Trick) -> HashMap<PlayerID, Vec<EffectiveSuit>> {
    let mut voids: HashMap<PlayerID, Vec<EffectiveSuit>> = HashMap::new();
    for trick in history.iter().chain(std::iter::once(current)) {
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

/// Every card played so far this hand, as far as we know.
pub(crate) fn played_cards(history: &[Trick], current: &Trick) -> Vec<Card> {
    history
        .iter()
        .chain(std::iter::once(current))
        .flat_map(|t| t.played_cards().iter())
        .flat_map(|pc| pc.cards.iter().copied())
        .filter(|c| *c != Card::Unknown)
        .collect()
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

/// Advice for the player called `name`, using only what that player can see. Works for card play
/// (when it's their turn) and for burying the kitty and calling friends.
pub fn hint(state: &GameState, name: &str) -> Option<Hint> {
    let me = state.player_id(name).ok()?;
    if !state.is_player(me) {
        return None;
    }
    let mut rng = StdRng::from_entropy();
    match state {
        GameState::Play(p) if p.trick.next_player() == Some(me) && !p.game_finished() => {
            let config = BotConfig::for_difficulty(Difficulty::Expert);
            play::play_hint(p, me, &Memory::default(), &config, &mut rng)
        }
        GameState::Exchange(e) => setup::exchange_hint(e, me),
        _ => None,
    }
}
