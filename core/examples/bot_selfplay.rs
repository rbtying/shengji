//! Plays whole games between bots inside the engine, with each bot seeing only its own redacted
//! view of the state (as it would over the websocket).
//!
//! Usage: bot_selfplay [players] [decks] [hands] [ms-per-move] [tractor|friends]

use std::time::{Duration, Instant};

use slog::{o, Discard, Logger};

use shengji_core::bot::{Bot, BotConfig};
use shengji_core::game_state::GameState;
use shengji_core::interactive::{Action, InteractiveGame};
use shengji_core::settings::GameModeSettings;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |i: usize, d: usize| args.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    let num_players = arg(1, 5);
    let num_decks = arg(2, 3);
    let num_hands = arg(3, 2);
    let ms = arg(4, 300) as u64;
    let tractor = args.get(5).map(|s| s == "tractor").unwrap_or(false);
    // "mixed": odd seats play by heuristic only, to measure what simulation adds.
    let mixed = args.get(6).map(|s| s == "mixed").unwrap_or(false);

    let logger = Logger::root(Discard, o!());
    let mut game = InteractiveGame::new();
    let mut ids = vec![];
    let mut bots = vec![];
    for i in 0..num_players {
        let name = format!("bot{i}");
        let (id, _) = game.register(name.clone()).unwrap();
        ids.push(id);
        bots.push(Bot::new(
            name,
            BotConfig {
                time_budget: Duration::from_millis(ms),
                start_new_games: true,
                heuristic_only: mixed && i % 2 == 1,
                ..BotConfig::default()
            },
        ));
    }
    let host = ids[0];
    game.interact(Action::SetNumDecks(Some(num_decks)), host, &logger)
        .unwrap();
    if !tractor {
        game.interact(
            Action::SetGameMode(GameModeSettings::FindingFriends { num_friends: None }),
            host,
            &logger,
        )
        .unwrap();
    }
    game.interact(Action::StartGame, host, &logger).unwrap();

    let mut hands_done = 0;
    let mut steps = 0usize;
    let mut plays = 0usize;
    let mut play_time = Duration::ZERO;
    let mut slowest = Duration::ZERO;
    let started = Instant::now();
    let mut was_play = false;

    'outer: loop {
        steps += 1;
        if steps > 200_000 {
            println!("step limit reached");
            break;
        }
        let mut acted = false;
        for i in 0..bots.len() {
            let bot = &mut bots[i];
            let view = game.dump_state_for_player(ids[i]).unwrap();
            bot.observe(&view);
            // HINT_TEST=1: print the hint's estimate for opening leads, to compare with results.
            if std::env::var("HINT_TEST").is_ok() {
                if let (GameState::Play(_), GameState::Play(fp)) =
                    (&view, game.dump_state().unwrap())
                {
                    let held: usize = fp
                        .hands()
                        .get(ids[i])
                        .map(|h| h.values().sum())
                        .unwrap_or(0);
                    if fp.trick().played_cards().is_empty()
                        && fp.next_player().ok() == Some(ids[i])
                        && held >= 20
                    {
                        if let Some(h) = shengji_core::bot::hint(&view, bot.name()) {
                            println!(
                                "HINT {} landlord_side={} :: {}",
                                bot.name(),
                                fp.landlords_team().contains(&ids[i]),
                                h.summary
                            );
                        }
                    }
                }
            }
            let t = Instant::now();
            let decision = bot.decide(&view);
            let dt = t.elapsed();
            let Some(d) = decision else { continue };
            let is_play = matches!(d.action, Action::PlayCards(_));
            if is_play {
                plays += 1;
                play_time += dt;
                slowest = slowest.max(dt);
            }
            let action_desc = format!("{:?}", d.action);
            match game.interact(d.action, ids[i], &logger) {
                Ok(msgs) => {
                    acted = true;
                    for (_, m) in msgs {
                        if !m.is_empty() && !m.contains("drew") {
                            println!("  [{}] {m}", bots[i].name());
                        }
                    }
                    if is_play && d.note.contains("samples") {
                        println!("    ({} -> {})", bots[i].name(), d.note);
                    }
                    if !is_play
                        && !action_desc.starts_with("DrawCard")
                        && !action_desc.starts_with("Move")
                        && !action_desc.starts_with("EndTrick")
                    {
                        println!("  {} -> {} [{}]", bots[i].name(), action_desc, d.note);
                    }
                }
                Err(e) => {
                    // Racing EndTrick / bids are expected to fail sometimes.
                    if !action_desc.starts_with("EndTrick") {
                        println!("  !! {} action {action_desc} failed: {e}", bots[i].name());
                    }
                }
            }
            // Let everyone see every state change.
            for (j, b) in bots.iter_mut().enumerate() {
                if j != i {
                    b.observe(&game.dump_state_for_player(ids[j]).unwrap());
                }
            }
            let state = game.dump_state().unwrap();
            let in_play = matches!(state, GameState::Play(_));
            if was_play && !in_play {
                hands_done += 1;
                println!("=== hand {hands_done} finished ===");
                if hands_done >= num_hands {
                    break 'outer;
                }
            }
            was_play = in_play;
        }
        if !acted {
            let state = game.dump_state().unwrap();
            if matches!(state, GameState::Initialize(_)) {
                // The host starts each hand, as a human would.
                game.interact(Action::StartGame, host, &logger).unwrap();
                continue;
            }
            println!(
                "STUCK: no bot wants to act. phase = {}",
                match state {
                    GameState::Initialize(_) => "initialize",
                    GameState::Draw(_) => "draw",
                    GameState::Exchange(_) => "exchange",
                    GameState::Play(_) => "play",
                }
            );
            break;
        }
    }

    let state = game.dump_state().unwrap();
    for p in state.players() {
        println!("{} level {:?}", p.name, p.level);
    }
    println!(
        "{hands_done} hands, {plays} card plays, avg {:.0} ms/play, slowest {} ms, total {:.1}s",
        play_time.as_millis() as f64 / plays.max(1) as f64,
        slowest.as_millis(),
        started.elapsed().as_secs_f64()
    );
}
