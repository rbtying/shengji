//! Connects one or more computer players to a shengji server room.
//!
//!   shengji-bot --room <16-char room name> [--names Ann,Bo,Cy] [--url ws://127.0.0.1:3030/api]
//!               [--think-ms 1500] [--samples 64] [--start-new-games] [--quiet]

use std::time::Duration;

use anyhow::{bail, Context, Error};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::time::{sleep_until, Instant};
use tokio_tungstenite::tungstenite::Message;

use shengji_core::bot::{Bot, BotConfig, Decision};
use shengji_core::interactive::Action;
use shengji_types::GameMessage;

#[derive(Serialize)]
struct JoinRoom<'a> {
    room_name: &'a str,
    name: &'a str,
    disable_compression: bool,
}

/// Mirrors the server's `UserMessage` (backend/src/serving_types.rs).
#[derive(Serialize)]
enum UserMessage {
    Action(Action),
    Ready,
}

#[derive(Clone)]
struct Options {
    url: String,
    room: String,
    names: Vec<String>,
    config: BotConfig,
    quiet: bool,
}

fn parse_args() -> Result<Options, Error> {
    let mut url = "ws://127.0.0.1:3030/api".to_string();
    let mut room = None;
    let mut names = vec!["Bot-Ann".to_string()];
    let mut config = BotConfig::default();
    let mut quiet = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut val = || args.next().context(format!("{a} needs a value"));
        match a.as_str() {
            "--url" => url = val()?,
            "--room" => room = Some(val()?),
            "--names" => names = val()?.split(',').map(|s| s.trim().to_string()).collect(),
            "--think-ms" => config.time_budget = Duration::from_millis(val()?.parse()?),
            "--samples" => config.samples = val()?.parse()?,
            "--threads" => config.threads = val()?.parse()?,
            "--start-new-games" => config.start_new_games = true,
            "--no-finish-tricks" => config.finish_tricks = false,
            "--quiet" => quiet = true,
            "-h" | "--help" => {
                println!(
                    "usage: shengji-bot --room <room> [--names A,B,C] [--url ws://127.0.0.1:3030/api]\n\
                     \x20      [--think-ms 1500] [--samples 64] [--threads N] [--start-new-games]\n\
                     \x20      [--no-finish-tricks] [--quiet]"
                );
                std::process::exit(0);
            }
            other => bail!("unknown argument {other}"),
        }
    }
    let room = room.context("--room is required (the 16-character code in the game's URL)")?;
    // Accept a full link like http://localhost:3030/#abcdefghijklmnop too.
    let room = room.rsplit('#').next().unwrap_or(&room).to_string();
    if room.len() != 16 {
        bail!("room name must be 16 characters, got {:?}", room);
    }
    for n in &names {
        if n.is_empty() || n.len() >= 32 {
            bail!("bot names must be 1-31 characters: {:?}", n);
        }
    }
    Ok(Options {
        url,
        room,
        names,
        config,
        quiet,
    })
}

async fn run_bot(opts: Options, name: String) -> Result<(), Error> {
    let (ws, _) = tokio_tungstenite::connect_async(opts.url.as_str())
        .await
        .with_context(|| format!("connecting to {}", opts.url))?;
    let (mut tx, mut rx) = ws.split();
    tx.send(Message::text(serde_json::to_string(&JoinRoom {
        room_name: &opts.room,
        name: &name,
        disable_compression: true,
    })?))
    .await?;
    println!("[{name}] joined room {}", opts.room);

    let mut bot = Bot::new(name.clone(), opts.config.clone());
    let mut pending: Option<(Decision, Instant)> = None;
    let mut last_state = None;
    let mut retry_at: Option<Instant> = None;
    let mut failures = 0u32;

    loop {
        let wake = pending
            .as_ref()
            .map(|(_, at)| *at)
            .or(retry_at)
            .unwrap_or_else(|| Instant::now() + Duration::from_secs(3600));
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
                    GameMessage::Error(e) => {
                        // Racing to finish a trick is expected; other errors get a retry.
                        if !e.contains("OutOfOrder") && !e.contains("out of order") {
                            eprintln!("[{name}] server rejected action: {e}");
                            failures += 1;
                            if failures <= 3 {
                                retry_at = Some(Instant::now() + Duration::from_secs(2));
                            }
                        }
                    }
                    _ => (),
                }
            }
            _ = sleep_until(wake) => {
                if let Some((d, _)) = pending.take() {
                    if !opts.quiet && !matches!(d.action, Action::DrawCard | Action::MoveCardToHand(_) | Action::MoveCardToKitty(_) | Action::EndTrick) {
                        println!("[{name}] {:?}  ({})", d.action, d.note);
                    }
                    tx.send(Message::text(serde_json::to_string(&UserMessage::Action(d.action))?)).await?;
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

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Error> {
    let opts = parse_args()?;
    let mut tasks = vec![];
    for (i, name) in opts.names.clone().into_iter().enumerate() {
        let opts = opts.clone();
        // Join one at a time so seats fill in the order given.
        tokio::time::sleep(Duration::from_millis(if i == 0 { 0 } else { 300 })).await;
        tasks.push(tokio::spawn(async move {
            if let Err(e) = run_bot(opts, name.clone()).await {
                eprintln!("[{name}] stopped: {e:#}");
            }
        }));
    }
    for t in tasks {
        let _ = t.await;
    }
    Ok(())
}
