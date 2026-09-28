//! A whole game with computer players, run in-process instead of on a server.
//!
//! It speaks the same messages as the server's websocket (`UserMessage` in, `GameMessage` out),
//! so a client can switch between online and offline play without changing anything else. The
//! host drives it: pass the human's messages to [`LocalGame::handle`], and call
//! [`LocalGame::tick`] whenever [`LocalGame::next_wake`] says a bot is due to act.

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use slog::{o, Discard, Logger};

use shengji_core::bot::{
    hint_against, review_play, summarize, Bot, BotConfig, Decision, Difficulty, PlayVerdict,
    RecordedPlay,
};
use shengji_core::game_state::GameState;
use shengji_core::interactive::{Action, InteractiveGame};
use shengji_mechanics::types::PlayerID;
use shengji_types::GameMessage;

const BOT_NAMES: [&str; 12] = [
    "Ann", "Bo", "Cy", "Di", "Ed", "Flo", "Gus", "Hal", "Ivy", "Jo", "Kit", "Lu",
];

/// Messages from the human player, as sent to the server's websocket.
#[derive(Deserialize)]
#[serde(untagged)]
enum Incoming {
    Join { name: String },
    User(UserMessage),
}

/// Mirrors the server's `UserMessage` (backend/src/serving_types.rs).
#[derive(Deserialize)]
enum UserMessage {
    Message(String),
    Action(Action),
    Kick(PlayerID),
    Beep,
    ReadyCheck,
    Ready,
    AddBot(Difficulty),
    RequestHint,
    RequestReview,
}

struct LocalBot {
    id: PlayerID,
    difficulty: Difficulty,
    bot: Bot,
    /// The game version this bot last looked at.
    seen: u64,
    /// What it plans to do, and when (ms, on the host's clock).
    pending: Option<(Decision, f64)>,
    /// Consecutive rejected actions for the current version.
    failures: u32,
}

/// Enough to resume an offline game later.
#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    state: GameState,
    human: String,
    bots: Vec<(String, Difficulty)>,
}

pub struct LocalGame {
    game: InteractiveGame,
    human: Option<(PlayerID, String)>,
    bots: Vec<LocalBot>,
    version: u64,
    logger: Logger,
    /// The human's plays this hand, with what they could see, for the after-hand review, and
    /// each one's analysis once done (`Some(None)`: nothing to review).
    plays: Vec<(RecordedPlay, Option<Option<PlayVerdict>>)>,
}

impl Default for LocalGame {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalGame {
    pub fn new() -> Self {
        LocalGame {
            game: InteractiveGame::new(),
            human: None,
            bots: vec![],
            version: 0,
            logger: Logger::root(Discard, o!()),
            plays: vec![],
        }
    }

    pub fn snapshot(&self) -> Result<Snapshot, Error> {
        let (_, human) = self
            .human
            .clone()
            .ok_or_else(|| anyhow!("nobody has joined"))?;
        Ok(Snapshot {
            state: self.game.dump_state()?,
            human,
            bots: self
                .bots
                .iter()
                .map(|b| (b.bot.name().to_string(), b.difficulty))
                .collect(),
        })
    }

    pub fn restore(snapshot: Snapshot) -> Result<Self, Error> {
        let game = InteractiveGame::new_from_state(snapshot.state);
        let state = game.dump_state()?;
        let human_id = state.player_id(&snapshot.human)?;
        let mut bots = vec![];
        for (name, difficulty) in snapshot.bots {
            if let Ok(id) = state.player_id(&name) {
                bots.push(new_bot(id, name, difficulty));
            }
        }
        Ok(LocalGame {
            game,
            human: Some((human_id, snapshot.human)),
            bots,
            version: 1,
            logger: Logger::root(Discard, o!()),
            plays: vec![],
        })
    }

    /// The current state as the human sees it.
    pub fn view(&self) -> Vec<GameMessage> {
        match &self.human {
            Some((id, _)) => self
                .game
                .dump_state_for_player(*id)
                .map(|state| vec![GameMessage::State { state }])
                .unwrap_or_default(),
            None => vec![],
        }
    }

    /// Handle one message from the human (JSON, as it would be sent to the server).
    pub fn handle(&mut self, json: &str) -> Vec<GameMessage> {
        match self.handle_inner(json) {
            Ok(mut msgs) => {
                msgs.extend(self.view());
                msgs
            }
            Err(e) => vec![GameMessage::Error(e.to_string())],
        }
    }

    fn handle_inner(&mut self, json: &str) -> Result<Vec<GameMessage>, Error> {
        let incoming: Incoming = serde_json::from_str(json)?;
        let user = match (incoming, &self.human) {
            (Incoming::Join { name }, None) => {
                let (id, msgs) = self.game.register(name.clone())?;
                self.human = Some((id, name));
                self.version += 1;
                return Ok(broadcasts(msgs));
            }
            // Re-joining (e.g. after a reload) just refreshes the view.
            (Incoming::Join { .. }, Some(_)) => return Ok(vec![]),
            (Incoming::User(_), None) => return Err(anyhow!("join the game first")),
            (Incoming::User(u), Some(_)) => u,
        };
        let (me, name) = self.human.clone().unwrap();
        match user {
            UserMessage::Action(action) => {
                let played = match &action {
                    Action::PlayCards(cards) | Action::PlayCardsWithHint(cards, _) => {
                        Some((self.game.dump_state_for_player(me)?, cards.clone()))
                    }
                    _ => None,
                };
                let starting = matches!(action, Action::StartGame);
                let msgs = self.game.interact(action, me, &self.logger)?;
                if starting {
                    self.plays.clear();
                }
                if let Some((view, cards)) = played {
                    self.plays.push((RecordedPlay { view, cards }, None));
                }
                self.version += 1;
                Ok(broadcasts(msgs))
            }
            UserMessage::RequestReview => {
                while self.idle_work(1500.0) {}
                let verdicts: Vec<PlayVerdict> = self
                    .plays
                    .iter()
                    .filter_map(|(_, v)| v.clone().flatten())
                    .collect();
                Ok(vec![GameMessage::Review {
                    review: Some(summarize(&verdicts)),
                }])
            }
            UserMessage::AddBot(difficulty) => {
                let state = self.game.dump_state()?;
                let taken: Vec<&str> = state
                    .players()
                    .iter()
                    .chain(state.observers().iter())
                    .map(|p| p.name.as_str())
                    .collect();
                let bot_name = BOT_NAMES
                    .iter()
                    .map(|n| format!("{n} ({})", difficulty.label()))
                    .find(|n| !taken.contains(&n.as_str()))
                    .ok_or_else(|| anyhow!("No more bot seats available"))?;
                let (id, msgs) = self.game.register(bot_name.clone())?;
                self.bots.push(new_bot(id, bot_name, difficulty));
                self.version += 1;
                Ok(broadcasts(msgs))
            }
            UserMessage::Kick(target) => {
                let msgs = self.game.kick(me, target)?;
                self.bots.retain(|b| b.id != target);
                self.version += 1;
                Ok(broadcasts(msgs))
            }
            UserMessage::RequestHint => {
                let view = self.game.dump_state_for_player(me)?;
                // With a table of same-level bots, the coach corrects its odds for their level.
                let level = self.bots.first().map(|b| b.difficulty);
                let opponents = level.filter(|l| self.bots.iter().all(|b| b.difficulty == *l));
                Ok(vec![GameMessage::Hint {
                    hint: hint_against(&view, &name, opponents),
                }])
            }
            UserMessage::Message(message) => Ok(vec![GameMessage::Message {
                from: name,
                message,
            }]),
            UserMessage::Beep | UserMessage::ReadyCheck | UserMessage::Ready => Ok(vec![]),
        }
    }

    /// Whether some of the human's plays still await review analysis.
    pub fn has_idle_work(&self) -> bool {
        self.plays.iter().any(|(_, v)| v.is_none())
    }

    /// Analyze one pending play for the after-hand review, within `budget_ms`. Hosts call this
    /// while nothing else is due (e.g. while the human is thinking). Returns false when idle.
    pub fn idle_work(&mut self, budget_ms: f64) -> bool {
        let Some((_, name)) = self.human.clone() else {
            return false;
        };
        match self.plays.iter_mut().find(|(_, v)| v.is_none()) {
            Some((rec, verdict)) => {
                let budget = std::time::Duration::from_millis(budget_ms.max(100.0) as u64);
                *verdict = Some(review_play(rec, &name, budget));
                true
            }
            None => false,
        }
    }

    /// When the next bot action is due, on the same clock passed to `tick`.
    /// Returns 0 when bots haven't yet looked at the latest state (tick right away).
    pub fn next_wake(&self) -> Option<f64> {
        if self.bots.iter().any(|b| b.seen != self.version) {
            return Some(0.0);
        }
        self.bots
            .iter()
            .filter_map(|b| b.pending.as_ref().map(|(_, at)| *at))
            .reduce(f64::min)
    }

    /// Let the bots catch up on the latest state, then take at most one due bot action.
    /// `now_ms` is any monotonic clock in milliseconds. May take a few seconds when a bot
    /// is thinking.
    pub fn tick(&mut self, now_ms: f64) -> Vec<GameMessage> {
        self.refresh(now_ms);
        let due = self
            .bots
            .iter()
            .enumerate()
            .filter_map(|(i, b)| b.pending.as_ref().map(|(_, at)| (i, *at)))
            .filter(|(_, at)| *at <= now_ms)
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let Some((i, _)) = due else { return vec![] };
        let (decision, _) = self.bots[i].pending.take().unwrap();
        let id = self.bots[i].id;
        match self.game.interact(decision.action, id, &self.logger) {
            Ok(msgs) => {
                self.version += 1;
                self.bots[i].failures = 0;
                let mut out = broadcasts(msgs);
                out.extend(self.view());
                out
            }
            Err(_) => {
                // Usually a harmless race (another bot already finished the trick). Look again
                // on the next tick, but don't retry forever.
                let b = &mut self.bots[i];
                b.failures += 1;
                if b.failures <= 3 {
                    b.seen = u64::MAX;
                }
                vec![]
            }
        }
    }

    fn refresh(&mut self, now_ms: f64) {
        for b in self.bots.iter_mut() {
            if b.seen == self.version {
                continue;
            }
            if b.seen != u64::MAX {
                b.failures = 0;
            }
            b.seen = self.version;
            let Ok(view) = self.game.dump_state_for_player(b.id) else {
                continue;
            };
            b.bot.observe(&view);
            b.pending = b.bot.decide(&view).map(|d| {
                let at = now_ms + d.delay.as_millis() as f64;
                (d, at)
            });
        }
    }
}

fn new_bot(id: PlayerID, name: String, difficulty: Difficulty) -> LocalBot {
    LocalBot {
        id,
        difficulty,
        bot: Bot::new(name, BotConfig::for_difficulty(difficulty)),
        seen: u64::MAX,
        pending: None,
        failures: 0,
    }
}

fn broadcasts(
    msgs: Vec<(shengji_core::interactive::BroadcastMessage, String)>,
) -> Vec<GameMessage> {
    msgs.into_iter()
        .map(|(data, message)| GameMessage::Broadcast { data, message })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use shengji_core::bot::hint;

    /// A human who always accepts the first suggested play, against three bots, for a full hand.
    #[test]
    fn plays_a_hand_offline() {
        let mut g = LocalGame::new();
        g.handle(r#"{"room_name":"offline-practice","name":"Me"}"#);
        for _ in 0..3 {
            let out = g.handle(r#"{"AddBot":"Easy"}"#);
            assert!(!out.iter().any(|m| matches!(m, GameMessage::Error(_))));
        }
        g.handle(r#"{"Action":"StartGame"}"#);
        let mut now = 0.0;
        let mut finished = false;
        for _ in 0..20_000 {
            now += 100.0;
            g.tick(now);
            let state = g
                .game
                .dump_state_for_player(g.human.as_ref().unwrap().0)
                .unwrap();
            let me = g.human.as_ref().unwrap().0;
            match &state {
                GameState::Draw(_) if state.next_player().ok() == Some(me) => {
                    g.handle(r#"{"Action":"DrawCard"}"#);
                }
                GameState::Play(p) if p.game_finished() => {
                    finished = true;
                    break;
                }
                GameState::Play(p) if p.trick().next_player() == Some(me) => {
                    let h = hint(&state, "Me").expect("a hint on my turn");
                    let msg = serde_json::json!({ "Action": { "PlayCards": h.cards } });
                    let out = g.handle(&msg.to_string());
                    assert!(
                        !out.iter().any(|m| matches!(m, GameMessage::Error(_))),
                        "hint play was rejected"
                    );
                }
                _ => (),
            }
        }
        assert!(finished, "the hand should finish");
        let review = g.handle(r#""RequestReview""#);
        let r = review
            .iter()
            .find_map(|m| match m {
                GameMessage::Review { review } => review.clone(),
                _ => None,
            })
            .expect("a review");
        assert!(r.reviewed > 0, "some plays should have had a choice");
        assert!(r.matched <= r.reviewed);
        // Following the coach's hints, most plays should match its review.
        assert!(
            r.matched * 2 >= r.reviewed,
            "{} of {} matched",
            r.matched,
            r.reviewed
        );
        println!(
            "review: {} of {} matched; {} moments",
            r.matched,
            r.reviewed,
            r.moments.len()
        );
        for m in &r.moments {
            println!("  {} :: {}", m.title, m.text);
        }
        let snap = serde_json::to_string(&g.snapshot().unwrap()).unwrap();
        let restored = LocalGame::restore(serde_json::from_str(&snap).unwrap()).unwrap();
        assert_eq!(restored.bots.len(), 3);
    }

    /// Calibration: does the coach's predicted win chance at your first play match how often
    /// your side actually wins when everyone follows the bots' play? Slow; run by hand with
    /// `cargo test -p local-game --release calibrate -- --ignored --nocapture`.
    /// CALIBRATE_BOTS picks the opponents' level (default Easy); CALIBRATE_HANDS the count.
    #[test]
    #[ignore]
    fn calibrate_hint_win_rate() {
        let hands: usize = std::env::var("CALIBRATE_HANDS")
            .ok()
            .and_then(|h| h.parse().ok())
            .unwrap_or(30);
        let level = std::env::var("CALIBRATE_BOTS").unwrap_or_else(|_| "Easy".into());
        let add_bot = format!(r#"{{"AddBot":"{level}"}}"#);
        // (predicted win chance, won) for each hand that offered a real choice.
        let mut results: Vec<(f64, bool)> = vec![];
        for n in 0..hands {
            let mut g = LocalGame::new();
            g.handle(r#"{"room_name":"offline-practice","name":"Me"}"#);
            for _ in 0..3 {
                g.handle(&add_bot);
            }
            g.handle(r#"{"Action":"StartGame"}"#);
            let me = g.human.as_ref().unwrap().0;
            let mut first: Option<f64> = None;
            let mut now = 0.0;
            for _ in 0..400_000 {
                now += 100.0;
                g.tick(now);
                let state = g.game.dump_state_for_player(me).unwrap();
                match &state {
                    GameState::Draw(_) if state.next_player().ok() == Some(me) => {
                        g.handle(r#"{"Action":"DrawCard"}"#);
                    }
                    GameState::Play(_) => {
                        let full = g.game.dump_state().unwrap();
                        let GameState::Play(p) = &full else {
                            unreachable!()
                        };
                        if p.game_finished() {
                            let (_, landlord_won, _) = p.finish_game().unwrap();
                            let mine = p.landlords_team().contains(&me);
                            let won = landlord_won == mine;
                            if let Some(f) = first {
                                results.push((f, won));
                            }
                            println!(
                                "hand {n}: predicted {:.0}%, {} ({})",
                                first.unwrap_or(0.0) * 100.0,
                                if won { "won" } else { "lost" },
                                if mine { "landlord side" } else { "attacking" }
                            );
                            break;
                        }
                        if p.trick().next_player() == Some(me) {
                            let h = hint(&state, "Me").expect("a hint on my turn");
                            // Forced plays carry no prediction; use the first real choice.
                            if first.is_none() {
                                first = h.options.first().map(|o| o.win_rate);
                            }
                            let msg = serde_json::json!({ "Action": { "PlayCards": h.cards } });
                            g.handle(&msg.to_string());
                        }
                    }
                    _ => (),
                }
            }
        }
        let k = results.len().max(1) as f64;
        let predicted: f64 = results.iter().map(|r| r.0).sum::<f64>() / k;
        let actual = results.iter().filter(|r| r.1).count() as f64 / k;
        println!(
            "{level} bots: predicted {:.0}%, actual {:.0}% over {} hands",
            predicted * 100.0,
            actual * 100.0,
            results.len()
        );
    }
}
