//! Connects one or more computer players to a shengji server room.
//!
//!   shengji-bot --room <16-char room name> [--names Ann,Bo,Cy] [--difficulty hard]
//!               [--url ws://127.0.0.1:3030/api] [--think-ms 1500] [--samples 64]
//!               [--start-new-games] [--quiet]

use std::time::Duration;

use anyhow::{bail, Context, Error};

use shengji_bot::run_bot;
use shengji_core::bot::{BotConfig, Difficulty};

#[derive(Clone)]
struct Options {
    url: String,
    room: String,
    names: Vec<String>,
    config: BotConfig,
    quiet: bool,
}

const USAGE: &str = "usage: shengji-bot --room <room> [--names A,B,C]
       [--difficulty beginner|easy|intermediate|hard|expert]
       [--url ws://127.0.0.1:3030/api] [--think-ms 1500] [--samples 64] [--threads N]
       [--start-new-games] [--no-finish-tricks] [--quiet]";

fn parse_args() -> Result<Options, Error> {
    let mut url = "ws://127.0.0.1:3030/api".to_string();
    let mut room = None;
    let mut names = vec!["Bot-Ann".to_string()];
    let mut difficulty = Difficulty::default();
    let mut think_ms = None;
    let mut samples = None;
    let mut threads = None;
    let mut start_new_games = false;
    let mut finish_tricks = true;
    let mut quiet = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut val = || args.next().context(format!("{a} needs a value"));
        match a.as_str() {
            "--url" => url = val()?,
            "--room" => room = Some(val()?),
            "--names" => names = val()?.split(',').map(|s| s.trim().to_string()).collect(),
            "--difficulty" => {
                let v = val()?.to_lowercase();
                difficulty = Difficulty::ALL
                    .into_iter()
                    .find(|d| d.label() == v)
                    .with_context(|| format!("unknown difficulty {v:?}"))?;
            }
            "--think-ms" => think_ms = Some(val()?.parse()?),
            "--samples" => samples = Some(val()?.parse()?),
            "--threads" => threads = Some(val()?.parse()?),
            "--start-new-games" => start_new_games = true,
            "--no-finish-tricks" => finish_tricks = false,
            "--quiet" => quiet = true,
            "-h" | "--help" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            other => bail!("unknown argument {other}\n{USAGE}"),
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
    let mut config = BotConfig::for_difficulty(difficulty);
    if let Some(ms) = think_ms {
        config.time_budget = Duration::from_millis(ms);
    }
    if let Some(s) = samples {
        config.samples = s;
    }
    if let Some(t) = threads {
        config.threads = t;
    }
    config.start_new_games = start_new_games;
    config.finish_tricks = finish_tricks;
    Ok(Options {
        url,
        room,
        names,
        config,
        quiet,
    })
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
            let result = run_bot(opts.url, opts.room, name.clone(), opts.config, opts.quiet).await;
            if let Err(e) = result {
                eprintln!("[{name}] stopped: {e:#}");
            }
        }));
    }
    for t in tasks {
        let _ = t.await;
    }
    Ok(())
}
