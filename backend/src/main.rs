#![deny(warnings)]

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use slog::Drain;
use tokio::prelude::*;
use tokio::sync::{mpsc, Mutex};
use warp::ws::{Message, WebSocket};
use warp::Filter;

use shengji_core::{game_state, interactive, types};

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

lazy_static::lazy_static! {
    static ref CARDS_JSON: CardsBlob = CardsBlob {
        cards: types::FULL_DECK.iter().map(|c| c.as_info()).collect()
    };

    static ref ROOT_LOGGER: slog::Logger = {
        #[cfg(not(feature = "dynamic"))]
        let drain = slog_bunyan::default(std::io::stdout());
        #[cfg(feature = "dynamic")]
        let drain = slog_term::FullFormat::new(slog_term::TermDecorator::new().build()).build();

        slog::Logger::root(
            std::sync::Mutex::new(drain).fuse(),
            slog::o!("version" => env!("CARGO_PKG_VERSION")),
        )
    };
}

#[derive(Clone, Serialize)]
struct CardsBlob {
    cards: Vec<types::CardInfo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct InMemoryStats {
    num_games_created: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct GameStats {
    num_games_created: usize,
    num_active_games: usize,
    num_players_online_now: usize,
}

struct GameState {
    game: interactive::InteractiveGame,
    users: HashMap<usize, UserState>,
    last_updated: Instant,
}

struct UserState {
    player_id: types::PlayerID,
    tx: mpsc::UnboundedSender<Result<Message, warp::Error>>,
}

impl UserState {
    pub fn send(&self, msg: &GameMessage) {
        if let Ok(s) = serde_json::to_string(msg) {
            let _ = self.tx.send(Ok(Message::text(s)));
        }
    }
}

type Games = Arc<Mutex<HashMap<String, GameState>>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinRoom {
    room_name: String,
    name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UserMessage {
    Message(String),
    Action(interactive::Message),
    Kick(types::PlayerID),
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameMessage {
    State {
        state: game_state::GameState,
        cards: Vec<types::Card>,
    },
    Message {
        from: String,
        message: String,
    },
    Broadcast {
        data: interactive::BroadcastMessage,
        message: String,
    },
    Error(String),
    Kicked,
}

const DUMP_PATH: &str = "/tmp/shengji_state.json";

#[tokio::main]
async fn main() {
    let mut game_state = HashMap::new();

    let init_logger = ROOT_LOGGER.new(slog::o!("dump_path" => DUMP_PATH));

    match tokio::fs::File::open(DUMP_PATH).await {
        Ok(mut f) => {
            let mut data = vec![];
            match f.read_to_end(&mut data).await {
                Ok(n) => {
                    slog::info!(init_logger, "Read state dump"; "num_bytes" => n);
                    match serde_json::from_slice::<HashMap<String, game_state::GameState>>(&data) {
                        Ok(dump) => {
                            for (room_name, game_dump) in dump {
                                game_state.insert(
                                    room_name,
                                    GameState {
                                        game: interactive::InteractiveGame::new_from_state(
                                            game_dump,
                                        ),
                                        users: HashMap::new(),
                                        last_updated: Instant::now(),
                                    },
                                );
                            }
                        }
                        Err(e) => {
                            slog::error!(init_logger, "Failed to deserialize file"; "error" => format!("{:?}", e));
                        }
                    }
                }
                Err(e) => {
                    slog::error!(init_logger, "Failed to read file"; "error" => format!("{:?}", e));
                }
            }
        }
        Err(e) => {
            slog::error!(init_logger, "Failed to open dump"; "error" => format!("{:?}", e));
        }
    }
    slog::info!(init_logger, "Loaded games from state dump"; "num_games" => game_state.len());

    let games = Arc::new(Mutex::new(game_state));
    let stats = Arc::new(Mutex::new(InMemoryStats::default()));

    let games = warp::any().map(move || (games.clone(), stats.clone()));

    let api = warp::path("api").and(warp::ws()).and(games.clone()).map(
        |ws: warp::ws::Ws, (games, stats)| {
            ws.on_upgrade(move |socket| user_connected(socket, games, stats))
        },
    );

    #[cfg(not(feature = "dynamic"))]
    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    #[cfg(not(feature = "dynamic"))]
    let rules = warp::path("rules").map(|| warp::reply::html(RULES_HTML));
    #[cfg(not(feature = "dynamic"))]
    let js = warp::path("main.js").map(|| {
        warp::http::Response::builder()
            .header("Content-Type", "text/javascript; charset=utf-8")
            .body(JS)
    });
    #[cfg(not(feature = "dynamic"))]
    let js_map = warp::path("main.js.map").map(|| {
        warp::http::Response::builder()
            .header("Content-Type", "text/javascript; charset=utf-8")
            .body(JS_MAP)
    });
    #[cfg(not(feature = "dynamic"))]
    let css = warp::path("style.css").map(|| {
        warp::http::Response::builder()
            .header("Content-Type", "text/css; charset=utf-8")
            .body(CSS)
    });
    #[cfg(not(feature = "dynamic"))]
    let worker_js = warp::path("timer-worker.js").map(|| {
        warp::http::Response::builder()
            .header("Content-Type", "text/javascript; charset=utf-8")
            .body(WORKER_JS)
    });

    #[cfg(feature = "dynamic")]
    let index = warp::path::end().and(warp::fs::file("../frontend/static/index.html"));
    #[cfg(feature = "dynamic")]
    let rules = warp::path("rules").and(warp::fs::file("../frontend/static/rules.html"));
    #[cfg(feature = "dynamic")]
    let js = warp::path("main.js").and(warp::fs::file("../frontend/dist/main.js"));
    #[cfg(feature = "dynamic")]
    let js_map = warp::path("main.js.map").and(warp::fs::file("../frontend/dist/main.js.map"));
    #[cfg(feature = "dynamic")]
    let css = warp::path("style.css").and(warp::fs::file("../frontend/static/style.css"));
    #[cfg(feature = "dynamic")]
    let worker_js =
        warp::path("timer-worker.js").and(warp::fs::file("../frontend/static/timer-worker.js"));

    let cards = warp::path("cards.json").map(|| warp::reply::json(&*CARDS_JSON));

    let dump_state = warp::path("full_state.json")
        .and(games.clone())
        .and_then(|(game, _)| dump_state(game));
    let game_stats = warp::path("stats")
        .and(games)
        .and_then(|(game, stats)| get_stats(game, stats));
    let routes = index
        .or(js)
        .or(js_map)
        .or(worker_js)
        .or(css)
        .or(cards)
        .or(api)
        .or(rules)
        .or(dump_state)
        .or(game_stats);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn dump_state(games: Games) -> Result<impl warp::Reply, warp::Rejection> {
    let mut state_dump: HashMap<String, game_state::GameState> = HashMap::new();
    let mut games = games.lock().await;
    // Drop all games where everyone is disconnected.
    games.retain(|_, game| {
        !game.users.is_empty() || game.last_updated.elapsed() <= Duration::from_secs(3600)
    });

    for (room_name, game_state) in games.iter() {
        if let Ok(snapshot) = game_state.game.dump_state() {
            state_dump.insert(room_name.clone(), snapshot);
        }
    }

    // Best-effort attempt to write the full state to disk, for fun.
    if let Ok(mut f) = tokio::fs::File::create(DUMP_PATH).await {
        if let Ok(json) = serde_json::to_vec(&state_dump) {
            let _ = f.write_all(&json).await;
            let _ = f.sync_all().await;
        }
    }

    Ok(warp::reply::json(&state_dump))
}

async fn get_stats(
    games: Games,
    stats: Arc<Mutex<InMemoryStats>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let games = games.lock().await;
    let stats = stats.lock().await;
    let InMemoryStats { num_games_created } = *stats;
    let num_players_online_now = games.values().map(|g| g.users.len()).sum::<usize>();
    Ok(warp::reply::json(&GameStats {
        num_games_created,
        num_players_online_now,
        num_active_games: games.len(),
    }))
}

#[allow(clippy::cognitive_complexity)]
async fn user_connected(ws: WebSocket, games: Games, stats: Arc<Mutex<InMemoryStats>>) {
    // Use a counter to assign a new unique ID for this user.
    let ws_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        let _ = result;
    }));

    let mut val = None;

    let tx_ = tx.clone();
    let send_to_user = move |msg| {
        if let Ok(msg) = serde_json::to_string(&msg) {
            if tx_.send(Ok(Message::text(msg))).is_err() {
                return false;
            }
        }
        true
    };

    while let Some(result) = user_ws_rx.next().await {
        if let Ok(msg) = result {
            match serde_json::from_slice::<JoinRoom>(msg.as_bytes()) {
                Ok(msg) if msg.room_name.len() == 16 && msg.name.len() < 32 => {
                    val = Some((msg.room_name, msg.name));
                    break;
                }
                Ok(_) => {
                    if !send_to_user(GameMessage::Error("invalid room or name".to_string())) {
                        break;
                    }
                }
                Err(err) => {
                    let err = GameMessage::Error(format!("couldn't deserialize message {:?}", err));
                    if !send_to_user(err) {
                        break;
                    }
                }
            }
        } else {
            break;
        }
    }

    if let Some((room, name)) = val {
        let player_id = {
            let mut g = games.lock().await;
            let game = g.entry(room.clone()).or_insert_with(|| GameState {
                game: interactive::InteractiveGame::new(),
                users: HashMap::new(),
                last_updated: Instant::now(),
            });
            game.last_updated = Instant::now();
            if game.users.is_empty() {
                let mut stats = stats.lock().await;
                stats.num_games_created += 1;
            }
            let (player_id, msgs) = match game.game.register(name.clone()) {
                Ok(player_id) => player_id,
                Err(err) => {
                    let err = GameMessage::Error(format!("couldn't register for game {:?}", err));
                    let _ = send_to_user(err);
                    return;
                }
            };
            game.users.insert(ws_id, UserState { player_id, tx });
            // send the updated game state to everyone!
            for user in game.users.values() {
                if let Ok((state, cards)) = game.game.dump_state_for_player(user.player_id) {
                    user.send(&GameMessage::State { state, cards });
                }

                for (data, message) in &msgs {
                    user.send(&GameMessage::Broadcast {
                        data: data.clone(),
                        message: message.clone(),
                    });
                }
            }
            player_id
        };
        let games2 = games.clone();

        while let Some(result) = user_ws_rx.next().await {
            match result {
                Ok(msg) => {
                    match serde_json::from_slice::<UserMessage>(msg.as_bytes()) {
                        Ok(UserMessage::Message(m)) => {
                            // Broadcast this msg to everyone
                            let g = games.lock().await;
                            if let Some(game) = g.get(&room) {
                                for user in game.users.values() {
                                    user.send(&GameMessage::Message {
                                        from: name.clone(),
                                        message: m.clone(),
                                    });
                                }
                            }
                        }
                        Ok(UserMessage::Kick(id)) => {
                            let mut g = games.lock().await;
                            if let Some(game) = g.get_mut(&room) {
                                match game.game.kick(id) {
                                    Ok(msgs) => {
                                        for user in game.users.values() {
                                            if user.player_id == id {
                                                user.send(&GameMessage::Kicked);
                                            } else if let Ok((state, cards)) =
                                                game.game.dump_state_for_player(user.player_id)
                                            {
                                                user.send(&GameMessage::State { state, cards });
                                            }
                                            for (data, message) in &msgs {
                                                user.send(&GameMessage::Broadcast {
                                                    data: data.clone(),
                                                    message: message.clone(),
                                                });
                                            }
                                        }
                                        game.users.retain(|_, u| u.player_id != id);
                                    }
                                    Err(err) => {
                                        let err = GameMessage::Error(format!("{}", err));
                                        if !send_to_user(err) {
                                            break;
                                        }
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                        Ok(UserMessage::Action(m)) => {
                            let mut g = games.lock().await;
                            if let Some(game) = g.get_mut(&room) {
                                match game.game.interact(m, player_id) {
                                    Ok(msgs) => {
                                        // send the updated game state to everyone!
                                        for user in game.users.values() {
                                            if let Ok((state, cards)) =
                                                game.game.dump_state_for_player(user.player_id)
                                            {
                                                for (data, message) in &msgs {
                                                    user.send(&GameMessage::Broadcast {
                                                        data: data.clone(),
                                                        message: message.clone(),
                                                    });
                                                }
                                                user.send(&GameMessage::State { state, cards });
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        // send the error back to the requester
                                        let err = GameMessage::Error(format!("{}", err));
                                        if !send_to_user(err) {
                                            break;
                                        }
                                    }
                                }
                            } else {
                                break;
                            }
                        }
                        Err(err) => {
                            let err = GameMessage::Error(format!(
                                "couldn't deserialize message {:?}",
                                err
                            ));
                            if !send_to_user(err) {
                                break;
                            }
                        }
                    }
                }
                Err(_) => {
                    break;
                }
            };
        }

        // user_ws_rx stream will keep processing as long as the user stays
        // connected. Once they disconnect, then...
        user_disconnected(room, ws_id, &games2).await;
    }
}

async fn user_disconnected(room: String, ws_id: usize, games: &Games) {
    // Stream closed up, so remove from the user list
    let mut g = games.lock().await;
    if let Some(game) = g.get_mut(&room) {
        game.users.remove(&ws_id);
        // If there is nobody connected anymore, drop the game entirely.
        if game.users.is_empty() {
            g.remove(&room);
        }
    }
}

#[cfg(not(feature = "dynamic"))]
static INDEX_HTML: &str = include_str!("../../frontend/static/index.html");
#[cfg(not(feature = "dynamic"))]
static RULES_HTML: &str = include_str!("../../frontend/static/rules.html");
#[cfg(not(feature = "dynamic"))]
static JS: &str = include_str!("../../frontend/dist/main.js");
#[cfg(not(feature = "dynamic"))]
static JS_MAP: &str = include_str!("../../frontend/dist/main.js.map");
#[cfg(not(feature = "dynamic"))]
static CSS: &str = include_str!("../../frontend/static/style.css");
#[cfg(not(feature = "dynamic"))]
static WORKER_JS: &str = include_str!("../../frontend/static/timer-worker.js");

#[cfg(test)]
mod tests {
    use super::CARDS_JSON;

    static CARDS_JSON_FROM_FILE: &str = include_str!("../../frontend/src/generated/cards.json");

    #[test]
    fn test_cards_json_compatibility() {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(
                &serde_json::to_string(&*CARDS_JSON).unwrap()
            )
            .unwrap(),
            serde_json::from_str::<serde_json::Value>(CARDS_JSON_FROM_FILE).unwrap(),
            "Run `yarn download-cards-json` with the backend running to sync the generated cards.json file"
        );
    }
}
