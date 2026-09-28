//! A whole game with computer players, run in-process instead of on a server.
//!
//! It speaks the same messages as the server's websocket (`UserMessage` in, `GameMessage` out),
//! so a client can switch between online and offline play without changing anything else. The
//! host drives it: pass the human's messages to [`LocalGame::handle`], and call
//! [`LocalGame::tick`] whenever [`LocalGame::next_wake`] says a bot is due to act.

use anyhow::{anyhow, Error};
use serde::{Deserialize, Serialize};
use slog::{o, Discard, Logger};

use shengji_core::bot::{hint, Bot, BotConfig, Decision, Difficulty};
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
                let msgs = self.game.interact(action, me, &self.logger)?;
                self.version += 1;
                Ok(broadcasts(msgs))
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
                Ok(vec![GameMessage::Hint {
                    hint: hint(&view, &name),
                }])
            }
            UserMessage::Message(message) => Ok(vec![GameMessage::Message {
                from: name,
                message,
            }]),
            UserMessage::Beep | UserMessage::ReadyCheck | UserMessage::Ready => Ok(vec![]),
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
        let snap = serde_json::to_string(&g.snapshot().unwrap()).unwrap();
        let restored = LocalGame::restore(serde_json::from_str(&snap).unwrap()).unwrap();
        assert_eq!(restored.bots.len(), 3);
    }
}
