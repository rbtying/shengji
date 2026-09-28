//! Seats computer players in a room on a shengji server over its websocket API.

use std::time::Duration;

use anyhow::{bail, Context, Error};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::time::{sleep_until, Instant};
use tokio_tungstenite::tungstenite::Message;

use shengji_core::bot::{Bot, BotConfig, Decision};
use shengji_core::interactive::Action;
use shengji_types::GameMessage;

/// How long a bot stays in a room with nothing happening before it leaves.
const IDLE_TIMEOUT: Duration = Duration::from_secs(2 * 60 * 60);

#[derive(Serialize)]
pub struct JoinRoom<'a> {
    room_name: &'a str,
    name: &'a str,
    disable_compression: bool,
}

/// Mirrors the server's `UserMessage` (backend/src/serving_types.rs).
#[derive(Serialize)]
pub enum UserMessage {
    Action(Action),
    Ready,
}

/// Join `room` as `name` and play until kicked, disconnected, or idle for too long.
pub async fn run_bot(
    url: String,
    room: String,
    name: String,
    config: BotConfig,
    quiet: bool,
) -> Result<(), Error> {
    let (ws, _) = tokio_tungstenite::connect_async(url.as_str())
        .await
        .with_context(|| format!("connecting to {url}"))?;
    let (mut tx, mut rx) = ws.split();
    tx.send(Message::text(serde_json::to_string(&JoinRoom {
        room_name: &room,
        name: &name,
        disable_compression: true,
    })?))
    .await?;
    if !quiet {
        println!("[{name}] joined room {room}");
    }

    let mut bot = Bot::new(name.clone(), config);
    let mut pending: Option<(Decision, Instant)> = None;
    let mut last_state = None;
    let mut retry_at: Option<Instant> = None;
    let mut failures = 0u32;
    let mut last_activity = Instant::now();

    loop {
        let wake = pending
            .as_ref()
            .map(|(_, at)| *at)
            .or(retry_at)
            .unwrap_or(last_activity + IDLE_TIMEOUT);
        tokio::select! {
            msg = rx.next() => {
                let data = match msg {
                    None => bail!("server closed the connection"),
                    Some(Err(e)) => bail!("websocket error: {e}"),
                    Some(Ok(Message::Binary(b))) => b,
                    Some(Ok(Message::Text(t))) => t.into_bytes(),
                    Some(Ok(Message::Close(_))) => bail!("server closed the connection"),
                    Some(Ok(_)) => continue,
                };
                let msg: GameMessage = match serde_json::from_slice(&data) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("[{name}] couldn't parse server message: {e}");
                        continue;
                    }
                };
                match msg {
                    GameMessage::State { state } => {
                        last_activity = Instant::now();
                        bot.observe(&state);
                        failures = 0;
                        retry_at = None;
                        let decision = tokio::task::block_in_place(|| bot.decide(&state));
                        pending = decision.map(|d| {
                            let at = Instant::now() + d.delay;
                            (d, at)
                        });
                        last_state = Some(state);
                    }
                    GameMessage::ReadyCheck { .. } => {
                        tx.send(Message::text(serde_json::to_string(&UserMessage::Ready)?)).await?;
                    }
                    GameMessage::Kicked { target } if target == name => {
                        println!("[{name}] was kicked");
                        return Ok(());
                    }
                    // Racing to finish a trick is expected; other errors get a retry.
                    GameMessage::Error(e)
                        if !e.contains("OutOfOrder") && !e.contains("out of order") =>
                    {
                        eprintln!("[{name}] server rejected action: {e}");
                        failures += 1;
                        if failures <= 3 {
                            retry_at = Some(Instant::now() + Duration::from_secs(2));
                        }
                    }
                    _ => (),
                }
            }
            _ = sleep_until(wake) => {
                if let Some((d, _)) = pending.take() {
                    if !quiet && !matches!(d.action, Action::DrawCard | Action::MoveCardToHand(_) | Action::MoveCardToKitty(_) | Action::EndTrick) {
                        println!("[{name}] {:?}  ({})", d.action, d.note);
                    }
                    tx.send(Message::text(serde_json::to_string(&UserMessage::Action(d.action))?)).await?;
                } else if retry_at.is_none() && last_activity.elapsed() >= IDLE_TIMEOUT {
                    if !quiet {
                        println!("[{name}] leaving idle room");
                    }
                    return Ok(());
                } else if retry_at.take().is_some() {
                    if let Some(state) = &last_state {
                        let decision = tokio::task::block_in_place(|| bot.decide(state));
                        pending = decision.map(|d| {
                            let at = Instant::now() + d.delay;
                            (d, at)
                        });
                    }
                }
            }
        }
    }
}
