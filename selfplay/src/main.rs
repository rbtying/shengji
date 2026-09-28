//! Bot self-play for tuning the coach and the bots. See README.md.
//!
//! `calibrate`: seat 0 follows the coach against bots of one level; every real choice records
//! the coach's win chance for its pick and whether seat 0's side went on to win the hand.
//!
//! `h2h`: Tractor with four bots of one level, one team simulating with smart rollouts and the
//! other with basic ones (swapping seats every hand); records each hand's level swing.
//!
//! Output is JSON lines, appended and flushed after every hand, so runs can be stopped and
//! resumed (and several runs' files simply concatenated).

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Error};
use serde_json::json;
use slog::{o, Discard, Logger};

use shengji_core::bot::{hint_with, on_landlord_side, Bot, BotConfig, Difficulty};
use shengji_core::game_state::GameState;
use shengji_core::interactive::{Action, InteractiveGame};
use shengji_mechanics::player::Player;
use shengji_mechanics::types::{PlayerID, Rank};

struct Args {
    cmd: String,
    bots: Difficulty,
    mode: String,
    hands: usize,
    out: String,
    jobs: usize,
    coach_smart: bool,
    bots_smart: bool,
}

fn parse_args() -> Result<Args, Error> {
    let mut it = std::env::args().skip(1);
    let cmd = it
        .next()
        .ok_or_else(|| anyhow!("usage: selfplay calibrate|h2h [options]"))?;
    let mut kv: HashMap<String, String> = HashMap::new();
    while let Some(k) = it.next() {
        let v = it.next().ok_or_else(|| anyhow!("missing value for {k}"))?;
        kv.insert(k.trim_start_matches('-').to_string(), v);
    }
    let get = |k: &str, d: &str| kv.get(k).cloned().unwrap_or_else(|| d.to_string());
    let bots: Difficulty = serde_json::from_value(json!(get("bots", "Expert")))?;
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    Ok(Args {
        cmd,
        bots,
        mode: get("mode", "tractor"),
        hands: get("hands", "20").parse()?,
        out: get("out", "selfplay.jsonl"),
        jobs: get("jobs", &(cores / 4).max(1).to_string()).parse()?,
        coach_smart: get("coach-smart", "false").parse()?,
        bots_smart: get("bots-smart", "false").parse()?,
    })
}

/// Players and the setup actions for a table (the app's Pagat preset).
fn table(mode: &str) -> Result<(usize, Vec<&'static str>), Error> {
    let friends = r#"{"SetGameMode":{"FindingFriends":{"num_friends":null}}}"#;
    let common = [
        r#"{"SetNumDecks":2}"#,
        r#"{"SetThrowPenalty":"TenPointsPerCard"}"#,
        r#"{"SetNextLandlordPolicy":"LandlordStaysIfWon"}"#,
    ];
    let (n, first, kitty) = match mode {
        "tractor" => (4, r#"{"SetGameMode":"Tractor"}"#, r#"{"SetKittySize":8}"#),
        "friends5" => (5, friends, r#"{"SetKittySize":8}"#),
        "friends6" => (6, friends, r#"{"SetKittySize":6}"#),
        m => bail!("unknown mode {m}"),
    };
    let mut actions = vec![first, kitty];
    actions.extend(common);
    Ok((n, actions))
}

fn level_value(p: &Player) -> i64 {
    let l = match p.level {
        Rank::Number(n) => n.as_u32() as i64,
        Rank::NoTrump => 15,
    };
    p.metalevel as i64 * 20 + l
}

/// Every seat's bot configuration; with a fixed sample count (no wall-clock cut-off) so results
/// don't depend on how busy the machine is.
fn config(d: Difficulty, smart: bool, threads: usize) -> BotConfig {
    let mut c = BotConfig::for_difficulty(d);
    c.smart_rollouts = smart;
    c.threads = threads;
    c.time_budget = Duration::from_secs(600);
    c
}

struct HandResult {
    /// (coach's win chance for its pick, cards left in hand, seat 0 on the landlord side then)
    predictions: Vec<(f64, usize, bool)>,
    seat0_landlord_side: bool,
    landlord_won: bool,
    /// Level change per seat.
    deltas: Vec<i64>,
    landlord_side: Vec<usize>,
}

/// Play one hand. `configs[i]` drives seat i; `coach` (if any) plays seat 0 by the coach's hints.
fn play_hand(
    mode: &str,
    configs: &[BotConfig],
    coach: Option<&BotConfig>,
) -> Result<HandResult, Error> {
    let logger = Logger::root(Discard, o!());
    let (n, setup) = table(mode)?;
    let mut g = InteractiveGame::new();
    let mut seats: Vec<(PlayerID, String, Bot)> = vec![];
    for (i, cfg) in configs.iter().enumerate().take(n) {
        let name = format!("P{i}");
        let (id, _) = g.register(name.clone())?;
        seats.push((id, name.clone(), Bot::new(name, cfg.clone())));
    }
    let host = seats[0].0;
    for a in setup {
        g.interact(serde_json::from_str::<Action>(a)?, host, &logger)?;
    }
    g.interact(Action::StartGame, host, &logger)?;
    let before: Vec<i64> = match g.dump_state()? {
        GameState::Draw(d) => seats
            .iter()
            .map(|(id, _, _)| {
                d.propagated()
                    .players()
                    .iter()
                    .find(|p| p.id == *id)
                    .map(level_value)
                    .unwrap_or(0)
            })
            .collect(),
        _ => vec![0; n],
    };

    let mut predictions = vec![];
    let mut stuck = 0;
    for _ in 0..100_000 {
        let full = g.dump_state()?;
        if let GameState::Play(p) = &full {
            if p.game_finished() {
                let (init, landlord_won, _) = p.finish_game()?;
                let deltas = seats
                    .iter()
                    .enumerate()
                    .map(|(i, (id, _, _))| {
                        init.propagated()
                            .players()
                            .iter()
                            .find(|pl| pl.id == *id)
                            .map(level_value)
                            .unwrap_or(0)
                            - before[i]
                    })
                    .collect();
                let team = p.landlords_team();
                return Ok(HandResult {
                    predictions,
                    seat0_landlord_side: team.contains(&seats[0].0),
                    landlord_won,
                    deltas,
                    landlord_side: (0..n).filter(|i| team.contains(&seats[*i].0)).collect(),
                });
            }
        }
        let turn = match &full {
            GameState::Play(p) => p.trick().next_player(),
            _ => None,
        };
        // The coach plays seat 0's cards.
        if let (Some(cfg), Some(t)) = (coach, turn) {
            if t == seats[0].0 {
                let view = g.dump_state_for_player(t)?;
                let h = hint_with(&view, &seats[0].1, cfg, None)
                    .ok_or_else(|| anyhow!("no hint on the coach's turn"))?;
                if let (Some(best), GameState::Play(vp)) = (h.options.first(), &view) {
                    if h.options.len() > 1 {
                        predictions.push((
                            best.win_rate,
                            vp.hands()
                                .get(t)
                                .map(|h| h.values().sum::<usize>())
                                .unwrap_or(0),
                            on_landlord_side(vp, t),
                        ));
                    }
                }
                seats[0].2.observe(&view);
                g.interact(Action::PlayCards(h.cards), t, &logger)?;
                continue;
            }
        }
        // Everyone else's quick actions first; the player to move thinks last.
        let mut order: Vec<usize> = (0..n).filter(|i| Some(seats[*i].0) != turn).collect();
        order.extend((0..n).filter(|i| Some(seats[*i].0) == turn));
        let mut chosen: Option<(Duration, usize, Action)> = None;
        for i in order {
            if Some(seats[i].0) == turn && chosen.is_some() {
                break;
            }
            let (id, _, bot) = &mut seats[i];
            let view = g.dump_state_for_player(*id)?;
            bot.observe(&view);
            if let Some(d) = bot.decide(&view) {
                if chosen.as_ref().map(|c| d.delay < c.0).unwrap_or(true) {
                    chosen = Some((d.delay, i, d.action));
                }
            }
        }
        let Some((_, i, action)) = chosen else {
            bail!("nobody can move");
        };
        if g.interact(action, seats[i].0, &logger).is_err() {
            stuck += 1;
            if stuck > 50 {
                bail!("bots keep making rejected moves");
            }
        } else {
            stuck = 0;
        }
    }
    bail!("hand did not finish")
}

fn main() -> Result<(), Error> {
    let args = parse_args()?;
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let threads = (cores / args.jobs).max(1);
    let out = Mutex::new(
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&args.out)?,
    );
    let next = AtomicUsize::new(0);
    let start = Instant::now();
    let label = serde_json::to_value(args.bots)?;

    std::thread::scope(|scope| {
        for _ in 0..args.jobs {
            scope.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::Relaxed);
                if k >= args.hands {
                    break;
                }
                let line = match args.cmd.as_str() {
                    "calibrate" => {
                        let (n, _) = table(&args.mode).unwrap();
                        let bots: Vec<BotConfig> = (0..n)
                            .map(|_| config(args.bots, args.bots_smart, threads))
                            .collect();
                        let coach = config(Difficulty::Expert, args.coach_smart, threads);
                        play_hand(&args.mode, &bots, Some(&coach)).map(|r| {
                            let won = r.landlord_won == r.seat0_landlord_side;
                            json!({
                                "kind": "calibrate", "bots": label, "mode": args.mode,
                                "coach_smart": args.coach_smart, "bots_smart": args.bots_smart,
                                "won": won, "final_landlord_side": r.seat0_landlord_side,
                                "predictions": r.predictions.iter()
                                    .map(|(p, t, s)| json!({"p": p, "cards_left": t, "landlord_side": s}))
                                    .collect::<Vec<_>>(),
                            })
                        })
                    }
                    "h2h" => {
                        // Smart on even seats for even hands, odd seats for odd hands.
                        let smart_seats = k % 2;
                        let bots: Vec<BotConfig> = (0..4)
                            .map(|i| config(args.bots, i % 2 == smart_seats, threads))
                            .collect();
                        play_hand("tractor", &bots, None).map(|r| {
                            let smart_delta: i64 = (0..4)
                                .filter(|i| i % 2 == smart_seats)
                                .map(|i| r.deltas[i])
                                .sum::<i64>()
                                / 2;
                            let basic_delta: i64 = (0..4)
                                .filter(|i| i % 2 != smart_seats)
                                .map(|i| r.deltas[i])
                                .sum::<i64>()
                                / 2;
                            let smart_landlord = r.landlord_side.iter().any(|i| i % 2 == smart_seats);
                            json!({
                                "kind": "h2h", "bots": label,
                                "smart_landlord": smart_landlord,
                                "landlord_won": r.landlord_won,
                                "smart_levels": smart_delta, "basic_levels": basic_delta,
                            })
                        })
                    }
                    c => Err(anyhow!("unknown command {c}")),
                };
                match line {
                    Ok(v) => {
                        let mut f = out.lock().unwrap();
                        let _ = writeln!(f, "{v}");
                        let _ = f.flush();
                        eprintln!(
                            "hand {} done ({:.0}s elapsed)",
                            k + 1,
                            start.elapsed().as_secs_f64()
                        );
                    }
                    Err(e) => eprintln!("hand {} failed: {e}", k + 1),
                }
            });
        }
    });
    Ok(())
}
