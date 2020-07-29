#![deny(warnings)]

use std::collections::HashMap;
use std::env;
use std::io::ErrorKind;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use slog::{error, info, o, Drain, Logger};
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

    static ref ROOT_LOGGER: Logger = {
        #[cfg(not(feature = "dynamic"))]
        let drain = slog_bunyan::default(std::io::stdout());
        #[cfg(feature = "dynamic")]
        let drain = slog_term::FullFormat::new(slog_term::TermDecorator::new().build()).build();

        Logger::root(
            slog_async::Async::new(drain.fuse()).build().fuse(),
            o!("commit" => env!("VERGEN_SHA_SHORT"))
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
    sha: &'static str,
}

struct GameState {
    game: interactive::InteractiveGame,
    users: HashMap<usize, UserState>,
    last_updated: Instant,
    monotonic_id: usize,
}

impl GameState {
    pub fn tracer(&mut self, logger: &Logger, room: &str, parent: Option<usize>) -> Logger {
        let elapsed = self.last_updated.elapsed();
        self.last_updated = Instant::now();
        self.monotonic_id += 1;
        if let Some(parent) = parent {
            logger.new(o!(
                "elapsed_ms" => elapsed.as_millis(),
                "span" => format!("{}:{}", room, self.monotonic_id),
                "parent_span" => format!("{}:{}", room, parent)
            ))
        } else {
            logger.new(o!(
                "elapsed_ms" => elapsed.as_millis(),
                "span" => format!("{}:{}", room, self.monotonic_id),
            ))
        }
    }
}

struct UserState {
    player_id: types::PlayerID,
    tx: mpsc::UnboundedSender<Result<Message, warp::Error>>,
}

impl UserState {
    pub async fn send(&self, msg: &GameMessage) -> bool {
        send_to_user(&self.tx, msg).await
    }
}

async fn send_to_user(
    tx: &'_ mpsc::UnboundedSender<Result<Message, warp::Error>>,
    msg: &GameMessage,
) -> bool {
    if let Ok(s) = serde_json::to_string(msg) {
        if let Ok(compressed) = zstd::stream::encode_all(&mut std::io::Cursor::new(s), 0) {
            return tx.send(Ok(Message::binary(compressed))).is_ok();
        }
    }
    false
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
    Beep,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameMessage {
    State {
        state: game_state::GameState,
    },
    Message {
        from: String,
        message: String,
    },
    Broadcast {
        data: interactive::BroadcastMessage,
        message: String,
    },
    Beep,
    Error(String),
    Kicked,
}

const DUMP_PATH: &str = "/tmp/shengji_state.json";

#[tokio::main]
async fn main() {
    let mut game_state = HashMap::new();

    let init_logger = ROOT_LOGGER.new(o!("dump_path" => DUMP_PATH));

    match tokio::fs::File::open(DUMP_PATH).await {
        Ok(mut f) => {
            let mut data = vec![];
            match f.read_to_end(&mut data).await {
                Ok(n) => {
                    info!(init_logger, "Read state dump"; "num_bytes" => n);
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
                                        monotonic_id: 0,
                                    },
                                );
                            }
                        }
                        Err(e) => {
                            error!(init_logger, "Failed to deserialize file"; "error" => format!("{:?}", e));
                        }
                    }
                }
                Err(e) => {
                    error!(init_logger, "Failed to read file"; "error" => format!("{:?}", e));
                }
            }
        }
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                error!(init_logger, "Failed to open dump"; "error" => format!("{:?}", e));
            }
        }
    }
    info!(init_logger, "Loaded games from state dump"; "num_games" => game_state.len());

    let games = Arc::new(Mutex::new(game_state));
    let stats = Arc::new(Mutex::new(InMemoryStats::default()));

    let games = warp::any().map(move || (games.clone(), stats.clone()));

    let api = warp::path("api").and(warp::ws()).and(games.clone()).map(
        |ws: warp::ws::Ws, (games, stats)| {
            ws.on_upgrade(move |socket| user_connected(socket, games, stats))
        },
    );

    let cards = warp::path("cards.json").map(|| warp::reply::json(&*CARDS_JSON));

    let websocket_host: Option<String> = std::env::var("WEBSOCKET_HOST").ok();
    let runtime_settings = warp::path("runtime.js").map(move || {
        warp::http::Response::builder()
            .header("Content-Type", "text/javascript; charset=utf-8")
            .body(match websocket_host.as_ref() {
                Some(s) => format!("window._WEBSOCKET_HOST = \"{}\";", s),
                None => "window._WEBSOCKET_HOST = null;".to_string(),
            })
    });

    let dump_state = warp::path("full_state.json")
        .and(games.clone())
        .and_then(|(game, _)| dump_state(game));
    let game_stats = warp::path("stats")
        .and(games)
        .and_then(|(game, stats)| get_stats(game, stats));

    #[cfg(feature = "dynamic")]
    let static_routes = warp::fs::dir("../frontend/dist").or(warp::fs::dir("../favicon"));
    #[cfg(not(feature = "dynamic"))]
    let static_routes =
        static_dir::static_dir!("../frontend/dist").or(static_dir::static_dir!("../favicon"));

    // TODO: Figure out if this can be redirected safely without this duplicate hax.
    #[cfg(feature = "dynamic")]
    let rules = warp::path("rules").and(warp::fs::file("../frontend/dist/rules.html"));
    #[cfg(not(feature = "dynamic"))]
    let rules = warp::path("rules")
        .map(|| warp::reply::html(include_str!("../../frontend/dist/rules.html")));

    let default_settings = warp::path("default_settings.json").and_then(default_propagated);
    let routes = runtime_settings
        .or(cards)
        .or(api)
        .or(dump_state)
        .or(game_stats)
        .or(default_settings)
        .or(static_routes)
        .or(rules);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn dump_state(games: Games) -> Result<impl warp::Reply, warp::Rejection> {
    let mut state_dump: HashMap<String, game_state::GameState> = HashMap::new();
    let mut games = games.lock().await;
    games.retain(|_, game| {
        // Drop all games where we haven't seen an update for over an hour.
        game.last_updated.elapsed() <= Duration::from_secs(3600)
    });

    let num_players_online_now = games.values().map(|g| g.users.len()).sum::<usize>();

    let mut num_players = 0;
    let mut num_observers = 0;
    let mut num_zombies = 0;

    for (room_name, game_state) in games.iter() {
        if let Ok(snapshot) = game_state.game.dump_state() {
            if !game_state.users.is_empty() {
                num_players += snapshot.players.len();
                num_observers = snapshot.observers.len();
            } else {
                num_zombies += 1;
            }
            state_dump.insert(room_name.clone(), snapshot);
        }
    }

    drop(games);

    let logger = ROOT_LOGGER.new(o!(
        "dump_path" => DUMP_PATH,
        "num_games" => state_dump.len(),
        "num_players" => num_players,
        "num_observers" => num_observers,
        "num_online_players" => num_players_online_now,
        "num_zombies" => num_zombies,
    ));

    // Best-effort attempt to write the full state to disk, for fun.
    match write_state_to_disk(&state_dump).await {
        Ok(()) => {
            info!(logger, "Dumped state to disk");
        }
        Err(e) => {
            error!(logger, "Failed to dump state to disk"; "error" => format!("{:?}", e));
        }
    }

    Ok(warp::reply::json(&state_dump))
}

async fn write_state_to_disk(
    state: &HashMap<String, game_state::GameState>,
) -> std::io::Result<()> {
    let mut f = tokio::fs::File::create(DUMP_PATH).await?;
    let json = serde_json::to_vec(state)?;
    f.write_all(&json).await?;
    f.sync_all().await?;

    Ok(())
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
        sha: env!("VERGEN_SHA"),
    }))
}

async fn default_propagated() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&game_state::PropagatedState::default()))
}

#[allow(clippy::cognitive_complexity)]
async fn user_connected(ws: WebSocket, games: Games, stats: Arc<Mutex<InMemoryStats>>) {
    // Use a counter to assign a new unique ID for this user.
    let ws_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    let logger = ROOT_LOGGER.new(o!("ws_id" => ws_id));
    info!(logger, "Websocket connection initialized");

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        let _ = result;
    }));

    let mut val = None;

    while let Some(result) = user_ws_rx.next().await {
        if let Ok(msg) = result {
            match serde_json::from_slice::<JoinRoom>(msg.as_bytes()) {
                Ok(msg) if msg.room_name.len() == 16 && msg.name.len() < 32 => {
                    val = Some((msg.room_name, msg.name));
                    break;
                }
                Ok(_) => {
                    if !send_to_user(&tx, &GameMessage::Error("invalid room or name".to_string()))
                        .await
                    {
                        break;
                    }
                }
                Err(err) => {
                    let err = GameMessage::Error(format!("couldn't deserialize message {:?}", err));
                    if !send_to_user(&tx, &err).await {
                        break;
                    }
                }
            }
        } else {
            break;
        }
    }

    if let Some((room, name)) = val {
        let logger = logger.new(o!("room" => room.clone(), "name" => name.clone()));

        let (player_id, join_span) = {
            let mut g = games.lock().await;
            let game = g.entry(room.clone()).or_insert_with(|| GameState {
                game: interactive::InteractiveGame::new(),
                users: HashMap::new(),
                last_updated: Instant::now(),
                monotonic_id: 0,
            });
            if game.users.is_empty() {
                info!(game.tracer(&logger, &room, None), "Creating new room");
                let mut stats = stats.lock().await;
                stats.num_games_created += 1;
            }

            let (player_id, msgs) = match game.game.register(name.clone()) {
                Ok(player_id) => player_id,
                Err(err) => {
                    error!(logger, "Failed to join room"; "error" => format!("{:?}", err));
                    let err = GameMessage::Error(format!("couldn't register for game {:?}", err));
                    let _ = send_to_user(&tx, &err).await;
                    return;
                }
            };
            info!(game.tracer(&logger, &room, Some(1)), "Joining room"; "player_id" => player_id.0);
            game.users.insert(
                ws_id,
                UserState {
                    player_id,
                    tx: tx.clone(),
                },
            );

            // if the same user joined before, remove its previous entry from the user list
            if !game.game.allows_multiple_sessions_per_user() {
                game.users
                    .retain(|id, user| user.player_id != player_id || *id == ws_id);
            }

            // send the updated game state to everyone!
            for user in game.users.values() {
                if let Ok(state) = game.game.dump_state_for_player(user.player_id) {
                    user.send(&GameMessage::State { state }).await;
                }

                for (data, message) in &msgs {
                    user.send(&GameMessage::Broadcast {
                        data: data.clone(),
                        message: message.clone(),
                    })
                    .await;
                }
            }
            (player_id, game.monotonic_id)
        };
        let games2 = games.clone();

        while let Some(result) = user_ws_rx.next().await {
            let result = match result {
                Ok(r) => r,
                Err(e) => {
                    error!(logger, "Failed to fetch message"; "error" => format!("{:?}", e));
                    break;
                }
            };
            let msg = match serde_json::from_slice::<UserMessage>(result.as_bytes()) {
                Ok(m) => m,
                Err(e) => {
                    error!(logger, "Failed to deserialize message"; "error" => format!("{:?}", e));
                    let err = GameMessage::Error(format!("couldn't deserialize message {:?}", e));
                    if !send_to_user(&tx, &err).await {
                        break;
                    } else {
                        continue;
                    }
                }
            };
            let mut g = games.lock().await;
            let game = if let Some(game) = g.get_mut(&room) {
                game
            } else {
                error!(logger, "Game not found");
                break;
            };
            let logger = game.tracer(&logger, &room, Some(join_span));
            match msg {
                UserMessage::Beep => match game.game.next_player() {
                    Ok(player_id) => {
                        for user in game.users.values() {
                            user.send(&GameMessage::Message {
                                from: name.clone(),
                                message: "BEEP".to_owned(),
                            })
                            .await;
                            if user.player_id == player_id {
                                user.send(&GameMessage::Beep).await;
                            }
                        }
                    }
                    Err(err) => {
                        let err = GameMessage::Error(format!("{}", err));
                        if !send_to_user(&tx, &err).await {
                            break;
                        }
                    }
                },
                UserMessage::Message(m) => {
                    // Broadcast this msg to everyone
                    for user in game.users.values() {
                        user.send(&GameMessage::Message {
                            from: name.clone(),
                            message: m.clone(),
                        })
                        .await;
                    }
                }
                UserMessage::Kick(id) => {
                    info!(logger, "Kicking user"; "other" => id.0);
                    match game.game.kick(id) {
                        Ok(msgs) => {
                            for user in game.users.values() {
                                if user.player_id == id {
                                    user.send(&GameMessage::Kicked).await;
                                } else if let Ok(state) =
                                    game.game.dump_state_for_player(user.player_id)
                                {
                                    user.send(&GameMessage::State { state }).await;
                                }
                                for (data, message) in &msgs {
                                    user.send(&GameMessage::Broadcast {
                                        data: data.clone(),
                                        message: message.clone(),
                                    })
                                    .await;
                                }
                            }
                            game.users.retain(|_, u| u.player_id != id);
                        }
                        Err(err) => {
                            let err = GameMessage::Error(format!("{}", err));
                            if !send_to_user(&tx, &err).await {
                                break;
                            }
                        }
                    }
                }
                UserMessage::Action(m) => {
                    match game.game.interact(m, player_id, &logger) {
                        Ok(msgs) => {
                            // send the updated game state to everyone!
                            for user in game.users.values() {
                                if let Ok(state) = game.game.dump_state_for_player(user.player_id) {
                                    for (data, message) in &msgs {
                                        user.send(&GameMessage::Broadcast {
                                            data: data.clone(),
                                            message: message.clone(),
                                        })
                                        .await;
                                    }
                                    user.send(&GameMessage::State { state }).await;
                                }
                            }
                        }
                        Err(err) => {
                            // send the error back to the requester
                            let err = GameMessage::Error(format!("{}", err));
                            if !send_to_user(&tx, &err).await {
                                break;
                            }
                        }
                    }
                }
            }
        }

        // user_ws_rx stream will keep processing as long as the user stays
        // connected. Once they disconnect, then...
        user_disconnected(room, ws_id, &games2, logger, join_span).await;
    }
}

async fn user_disconnected(
    room: String,
    ws_id: usize,
    games: &Games,
    logger: slog::Logger,
    parent: usize,
) {
    // Stream closed up, so remove from the user list
    let mut g = games.lock().await;
    if let Some(game) = g.get_mut(&room) {
        game.users.remove(&ws_id);
        // If there is nobody connected anymore, drop the game entirely.
        if game.users.is_empty() {
            info!(game.tracer(&logger, &room, Some(parent)), "Removing empty room"; "room" => room.clone());
            g.remove(&room);
        }
    }
    drop(g);
    info!(logger, "Websocket disconnected";
        "room" => room,
        "parent_span" => format!("{}:{}", room, parent),
        "span" => format!("{}:ws_{}", room, ws_id)
    );
}

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
