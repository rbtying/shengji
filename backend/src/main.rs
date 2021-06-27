#![deny(warnings)]

use std::collections::HashMap;
use std::env;
use std::io::{self, ErrorKind};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::SinkExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use slog::{debug, error, info, o, Drain, Logger};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot, Mutex};
use warp::ws::{Message, WebSocket};
use warp::Filter;

use shengji_core::{game_state, interactive, settings, types};
use shengji_types::{GameMessage, ZSTD_ZSTD_DICT};

mod storage;
use storage::{HashMapStorage, State, Storage};

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

        let version = std::env::var("VERSION").unwrap_or_else(|_| env!("VERGEN_SHA_SHORT").to_string());

        Logger::root(
            slog_async::Async::new(drain.fuse()).build().fuse(),
            o!("version" => version)
        )
    };

    static ref ZSTD_COMPRESSOR: std::sync::Mutex<zstd::block::Compressor> = {
        let mut decomp = zstd::block::Decompressor::new();
        // default zstd dictionary size is 112_640
        let comp = zstd::block::Compressor::with_dict(decomp.decompress(ZSTD_ZSTD_DICT, 112_640).unwrap());
        std::sync::Mutex::new(comp)
    };

    static ref VERSION: String = {
        std::env::var("VERSION").unwrap_or_else(|_| env!("VERGEN_SHA").to_string())
    };

    static ref DUMP_PATH: String = {
        std::env::var("DUMP_PATH").unwrap_or_else(|_| "/tmp/shengji_state.json".to_string())
    };
    static ref MESSAGE_PATH: String = {
        std::env::var("MESSAGE_PATH").unwrap_or_else(|_| "/tmp/shengji_messages.json".to_string())
    };

}

#[derive(Clone, Serialize)]
struct CardsBlob {
    cards: Vec<types::CardInfo>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct InMemoryStats {
    num_games_created: usize,
    header_messages: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct GameStats<'a> {
    num_games_created: usize,
    num_active_games: usize,
    num_players_online_now: usize,
    sha: &'a str,
}

#[derive(Serialize, Deserialize, Clone)]
struct VersionedGame {
    room_name: Vec<u8>,
    game: shengji_core::game_state::GameState,
    monotonic_id: u64,
}

impl State for VersionedGame {
    type Message = GameMessage;

    fn version(&self) -> u64 {
        self.monotonic_id
    }

    fn key(&self) -> &[u8] {
        &self.room_name
    }

    fn new_from_key(key: Vec<u8>) -> Self {
        VersionedGame {
            room_name: key,
            game: shengji_core::game_state::GameState::Initialize(
                shengji_core::game_state::InitializePhase::new(),
            ),
            monotonic_id: 0,
        }
    }
}

// impl GameState {
//     pub fn tracer(&mut self, logger: &Logger, room: &str, parent: Option<usize>) -> Logger {
//         let elapsed = self.last_updated.elapsed();
//         self.last_updated = Instant::now();
//         self.monotonic_id += 1;
//         if let Some(parent) = parent {
//             logger.new(o!(
//                 "elapsed_ms" => elapsed.as_millis(),
//                 "span" => format!("{}:{}", room, self.monotonic_id),
//                 "parent_span" => format!("{}:{}", room, parent)
//             ))
//         } else {
//             logger.new(o!(
//                 "elapsed_ms" => elapsed.as_millis(),
//                 "span" => format!("{}:{}", room, self.monotonic_id),
//             ))
//         }
//     }
// }

#[derive(Clone)]
struct UserState {
    player_id: types::PlayerID,
    tx: mpsc::UnboundedSender<Message>,
}

impl UserState {
    pub async fn send(&self, msg: &GameMessage) -> bool {
        send_to_user(&self.tx, msg).await
    }
}

async fn send_to_user(tx: &'_ mpsc::UnboundedSender<Message>, msg: &GameMessage) -> bool {
    if let Ok(j) = serde_json::to_vec(&msg) {
        if let Ok(s) = ZSTD_COMPRESSOR.lock().unwrap().compress(&j, 0) {
            return tx.send(Message::binary(s)).is_ok();
        }
    }
    false
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JoinRoom {
    room_name: String,
    name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UserMessage {
    Message(String),
    Action(interactive::Action),
    Kick(types::PlayerID),
    Beep,
    ReadyCheck,
    Ready,
}

#[tokio::main]
async fn main() {
    let backend_storage = HashMapStorage::new();
    let mut num_games_loaded = 0usize;

    let init_logger = ROOT_LOGGER.new(o!("dump_path" => &*DUMP_PATH));

    match try_read_file::<HashMap<String, serde_json::Value>>(&*DUMP_PATH).await {
        Ok(dump) => {
            for (room_name, game_dump) in dump {
                match serde_json::from_value(game_dump) {
                    Ok(game_dump) => {
                        let upsert_result = backend_storage
                            .clone()
                            .put(VersionedGame {
                                room_name: room_name.as_bytes().to_vec(),
                                game: game_dump,
                                monotonic_id: 1,
                            })
                            .await;
                        if let Err(e) = upsert_result {
                            error!(init_logger, "Failed to upsert initial game state"; "error" => format!("{:?}", e), "room" => room_name);
                        } else {
                            num_games_loaded += 1;
                        }
                    }
                    Err(e) => {
                        error!(init_logger, "Failed to open per-game dump"; "error" => format!("{:?}", e), "room" => room_name);
                    }
                }
            }
        }
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                error!(init_logger, "Failed to open dump"; "error" => format!("{:?}", e));
            }
        }
    }

    info!(init_logger, "Loaded games from state dump"; "num_games" => num_games_loaded);

    let stats = Arc::new(Mutex::new(InMemoryStats::default()));

    match try_read_file::<Vec<String>>(&*MESSAGE_PATH).await {
        Ok(messages) => {
            let mut stats = stats.lock().await;
            stats.header_messages = messages;
        }
        Err(e) => {
            if e.kind() != ErrorKind::NotFound {
                error!(init_logger, "Failed to open message file"; "error" => format!("{:?}", e));
            }
        }
    }

    let periodic_task = periodically_dump_state(backend_storage.clone(), stats.clone());

    let games_filter = warp::any().map(move || (backend_storage.clone(), stats.clone()));

    let api = warp::path("api")
        .and(warp::ws())
        .and(games_filter.clone())
        .map(|ws: warp::ws::Ws, (backend_storage, stats)| {
            ws.on_upgrade(move |socket| user_connected(socket, backend_storage, stats))
        });

    let cards = warp::path("cards.json").map(|| warp::reply::json(&*CARDS_JSON));

    let websocket_host: Option<String> = std::env::var("WEBSOCKET_HOST").ok();
    let runtime_settings = warp::path("runtime.js").map(move || {
        warp::http::Response::builder()
            .header("Content-Type", "text/javascript; charset=utf-8")
            .body(match websocket_host.as_ref() {
                Some(s) => format!(
                    "window._WEBSOCKET_HOST = \"{}\";window._VERSION = \"{}\";",
                    s, *VERSION,
                ),
                None => format!(
                    "window._WEBSOCKET_HOST = null;window._VERSION = \"{}\";",
                    *VERSION
                ),
            })
    });

    let dump_state = warp::path("full_state.json")
        .and(games_filter.clone())
        .and_then(|(backend_storage, stats)| dump_state(backend_storage, stats));
    let game_stats = warp::path("stats")
        .and(games_filter)
        .and_then(|(backend_storage, stats)| get_stats(backend_storage, stats));

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

    let serve_task = warp::serve(routes).run(([0, 0, 0, 0], 3030));

    tokio::select! {
        () = periodic_task => unreachable!(),
        () = serve_task => {
            info!(init_logger, "Shutting down")
        },
    }
}

async fn try_read_file<M: serde::de::DeserializeOwned>(path: &'_ str) -> Result<M, io::Error> {
    let mut f = tokio::fs::File::open(path).await?;
    let mut data = vec![];
    f.read_to_end(&mut data).await?;
    Ok(serde_json::from_slice(&data)?)
}

async fn periodically_dump_state<S: Storage<VersionedGame, E>, E>(
    backend_storage: S,
    stats: Arc<Mutex<InMemoryStats>>,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        let _ = dump_state(backend_storage.clone(), stats.clone()).await;
    }
}

async fn dump_state<S: Storage<VersionedGame, E>, E>(
    _backend_storage: S,
    _stats: Arc<Mutex<InMemoryStats>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let state_dump: HashMap<String, game_state::GameState> = HashMap::new();

    // let header_messages = try_read_file::<Vec<String>>(&*MESSAGE_PATH)
    //     .await
    //     .unwrap_or_default();
    // let send_header_messages = {
    //     let mut stats = stats.lock().await;
    //     if stats.header_messages != header_messages {
    //         stats.header_messages = header_messages.clone();
    //         true
    //     } else {
    //         false
    //     }
    // };

    // let mut games = games.lock().await;
    // games.retain(|_, game| {
    //     // Drop all games where we haven't seen an update for over an hour.
    //     game.last_updated.elapsed() <= Duration::from_secs(3600)
    // });

    // let num_players_online_now = games.values().map(|g| g.users.len()).sum::<usize>();

    // let mut num_players = 0;
    // let mut num_observers = 0;
    // let mut num_zombies = 0;

    // for (room_name, game_state) in games.iter() {
    //     if let Ok(snapshot) = game_state.game.dump_state() {
    //         if !game_state.users.is_empty() {
    //             num_players += snapshot.players().len();
    //             num_observers = snapshot.observers().len();
    //         } else {
    //             num_zombies += 1;
    //         }
    //         state_dump.insert(room_name.clone(), snapshot);
    //     }
    // }
    // if send_header_messages {
    //     let msg = GameMessage::Header {
    //         messages: header_messages,
    //     };
    //     for (_, game) in games.iter() {
    //         for user in game.users.values() {
    //             let _ = user.send(&msg).await;
    //         }
    //     }
    // }

    // drop(games);

    // let logger = ROOT_LOGGER.new(o!(
    //     "dump_path" => &*DUMP_PATH,
    //     "num_games" => state_dump.len(),
    //     "num_players" => num_players,
    //     "num_observers" => num_observers,
    //     "num_online_players" => num_players_online_now,
    //     "num_zombies" => num_zombies,
    // ));

    // Best-effort attempt to write the full state to disk, for fun.
    // match write_state_to_disk(&state_dump).await {
    //     Ok(()) => {
    //         info!(logger, "Dumped state to disk");
    //     }
    //     Err(e) => {
    //         error!(logger, "Failed to dump state to disk"; "error" => format!("{:?}", e));
    //     }
    // }

    Ok(warp::reply::json(&state_dump))
}

#[allow(unused)]
async fn write_state_to_disk(
    state: &HashMap<String, game_state::GameState>,
) -> std::io::Result<()> {
    let mut f = tokio::fs::File::create(&*DUMP_PATH).await?;
    let json = serde_json::to_vec(state)?;
    f.write_all(&json).await?;
    f.sync_all().await?;

    Ok(())
}

async fn get_stats<S: Storage<VersionedGame, E>, E>(
    _backend_storage: S,
    _stats: Arc<Mutex<InMemoryStats>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // let games = games.lock().await;
    // let stats = stats.lock().await;
    // let InMemoryStats {
    //     num_games_created,
    //     header_messages: _,
    // } = *stats;
    // let num_players_online_now = games.values().map(|g| g.users.len()).sum::<usize>();
    // Ok(warp::reply::json(&GameStats {
    //     num_games_created,
    //     num_players_online_now,
    //     num_active_games: games.len(),
    //     sha: &*VERSION,
    // }))
    Ok(warp::reply::json(&GameStats::default()))
}

async fn default_propagated() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&settings::PropagatedState::default()))
}

#[allow(clippy::cognitive_complexity)]
async fn user_connected<S: Storage<VersionedGame, E>, E: std::fmt::Debug>(
    ws: WebSocket,
    backend_storage: S,
    stats: Arc<Mutex<InMemoryStats>>,
) {
    // Use a counter to assign a new unique ID for this user.
    let ws_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    let logger = ROOT_LOGGER.new(o!("ws_id" => ws_id));
    info!(logger, "Websocket connection initialized");

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, mut rx) = mpsc::unbounded_channel();
    tokio::task::spawn(async move {
        while let Some(v) = rx.recv().await {
            let _ = user_ws_tx.send(v).await;
        }
    });

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
        let logger =
            logger.new(o!("room" => room.clone(), "name" => name.clone(), "ws_id" => ws_id));

        let mut subscription = match backend_storage
            .clone()
            .subscribe(room.as_bytes().to_vec())
            .await
        {
            Ok(sub) => sub,
            Err(e) => {
                let _ = send_to_user(
                    &tx,
                    &GameMessage::Error(format!("Failed to join room: {:?}", e)),
                )
                .await;
                return;
            }
        };

        // Subscribe to messages for the room
        let tx_ = tx.clone();
        let logger_ = logger.clone();
        let name_ = name.clone();
        tokio::task::spawn(async move {
            debug!(logger_, "Subscribed to messages");
            while let Some(v) = subscription.recv().await {
                let should_send = match &v {
                    GameMessage::State { .. }
                    | GameMessage::Broadcast { .. }
                    | GameMessage::Message { .. }
                    | GameMessage::Error(_)
                    | GameMessage::Header { .. } => true,
                    GameMessage::Beep { target } | GameMessage::Kicked { target } => {
                        *target == name_
                    }
                    GameMessage::ReadyCheck { from } => *from != name_,
                };
                if should_send {
                    slog::debug!(logger_, "Sending message to user"; "msg" => format!("{:?}", v));
                    if !send_to_user(&tx_, &v).await {
                        break;
                    }
                }
            }
        });

        let logger_ = logger.clone();
        let (player_id_tx, player_id_rx) = oneshot::channel();
        let name_ = name.clone();
        execute_operation(
            tx.clone(),
            &room,
            backend_storage.clone(),
            move |g, version| {
                let (assigned_player_id, register_msgs) = g.register(name_)?;
                info!(logger_, "Joining room"; "player_id" => assigned_player_id.0);
                player_id_tx
                    .send((assigned_player_id, version))
                    .map_err(|_| anyhow::anyhow!("Couldn't send player ID back".to_owned()))?;
                Ok(register_msgs
                    .into_iter()
                    .map(|(data, message)| GameMessage::Broadcast { data, message })
                    .collect())
            },
            "register game",
        )
        .await;

        // let mut g = games.lock().await;
        // let game = g.entry(room.clone()).or_insert_with(|| GameState {
        //     game: interactive::InteractiveGame::new(),
        //     users: HashMap::new(),
        //     last_updated: Instant::now(),
        //     monotonic_id: 0,
        // });

        let header_messages = {
            let stats = stats.lock().await;
            stats.header_messages.clone()
        };
        let _ = send_to_user(
            &tx,
            &GameMessage::Header {
                messages: header_messages,
            },
        )
        .await;

        if let Ok((player_id, join_span)) = player_id_rx.await {
            let logger = logger.new(o!("player_id" => player_id.0));
            info!(logger, "Successfully registered user");
            // Handle the main game loop
            let caller = UserState { player_id, tx };

            while let Some(result) = user_ws_rx.next().await {
                let result = match result {
                    Ok(r) => r,
                    Err(e) => {
                        error!(logger, "Failed to fetch message"; "error" => format!("{:?}", e));
                        break;
                    }
                };
                match serde_json::from_slice::<UserMessage>(result.as_bytes()) {
                    Ok(msg) => {
                        if let Err(e) = handle_user_action_2(
                            logger.clone(),
                            caller.clone(),
                            &room,
                            name.clone(),
                            backend_storage.clone(),
                            msg,
                        )
                        .await
                        {
                            let _ = caller
                                .send(&GameMessage::Error(format!("Unexpected error {:?}", e)))
                                .await;
                        }
                    }
                    Err(e) => {
                        error!(logger, "Failed to deserialize message"; "error" => format!("{:?}", e));
                        let _ = caller
                            .send(&GameMessage::Error(format!(
                                "couldn't deserialize message {:?}",
                                e
                            )))
                            .await;
                    }
                }
            }

            // user_ws_rx stream will keep processing as long as the user stays
            // connected. Once they disconnect, then...
            user_disconnected(room, ws_id, backend_storage, logger, join_span).await;
        } else {
            return;
        }

        // if game.users.is_empty() {
        //     info!(game.tracer(&logger, &room, None), "Creating new room");
        //     let mut stats = stats.lock().await;
        //     stats.num_games_created += 1;
        // }

        // let (player_id, msgs) = match game.game.register(name.clone()) {
        //     Ok(player_id) => player_id,
        //     Err(err) => {
        //         error!(logger, "Failed to join room"; "error" => format!("{:?}", err));
        //         let err = GameMessage::Error(format!("couldn't register for game {:?}", err));
        //         let _ = send_to_user(&tx, &err).await;
        //         return;
        //     }
        // };

        // info!(game.tracer(&logger, &room, Some(1)), "Joining room"; "player_id" => player_id.0);
        // game.users.insert(
        //     ws_id,
        //     UserState {
        //         player_id,
        //         tx: tx.clone(),
        //     },
        // );

        // if the same user joined before, remove its previous entry from the user list
        // if !game.game.allows_multiple_sessions_per_user() {
        //     game.users
        //         .retain(|id, user| user.player_id != player_id || *id == ws_id);
        // }

        // send the updated game state to everyone!
        // for user in game.users.values() {
        //     if let Ok(state) = game.game.dump_state_for_player(user.player_id) {
        //         user.send(&GameMessage::State { state }).await;
        //     }

        //     for (data, message) in &msgs {
        //         user.send(&GameMessage::Broadcast {
        //             data: data.clone(),
        //             message: message.clone(),
        //         })
        //         .await;
        //     }
        // }
    }
}

enum EitherError<E> {
    E(E),
    E2(anyhow::Error),
}
impl<E> From<E> for EitherError<E> {
    fn from(e: E) -> Self {
        EitherError::E(e)
    }
}

async fn execute_operation<S, E, F>(
    user_tx: mpsc::UnboundedSender<Message>,
    room_name: &str,
    backend_storage: S,
    operation: F,
    action_description: &'static str,
) -> bool
where
    S: Storage<VersionedGame, E>,
    F: FnOnce(&mut interactive::InteractiveGame, u64) -> Result<Vec<GameMessage>, anyhow::Error>
        + Send
        + 'static,
{
    let room_name_ = room_name.as_bytes().to_vec();

    let res = backend_storage
        .execute_operation_with_messages::<EitherError<E>, _>(room_name_, move |versioned_game| {
            let mut g = interactive::InteractiveGame::new_from_state(versioned_game.game);
            let mut msgs =
                operation(&mut g, versioned_game.monotonic_id).map_err(EitherError::E2)?;
            let state = g.into_state();
            msgs.push(GameMessage::State {
                state: state.clone(),
            });
            Ok((
                VersionedGame {
                    room_name: versioned_game.room_name,
                    game: state,
                    monotonic_id: versioned_game.monotonic_id + 1,
                },
                msgs,
            ))
        })
        .await;
    match res {
        Ok(_) => true,
        Err(EitherError::E(_)) => {
            let err = GameMessage::Error(format!("Failed to {}", action_description));
            let _ = send_to_user(&user_tx, &err).await;
            false
        }
        Err(EitherError::E2(msg)) => {
            let err = GameMessage::Error(format!("Failed to {}: {}", action_description, msg));
            let _ = send_to_user(&user_tx, &err).await;
            false
        }
    }
}

async fn execute_immutable_operation<S, E, F>(
    user_tx: mpsc::UnboundedSender<Message>,
    room_name: &str,
    backend_storage: S,
    operation: F,
    action_description: &'static str,
) -> bool
where
    S: Storage<VersionedGame, E>,
    F: FnOnce(&interactive::InteractiveGame, u64) -> Result<Vec<GameMessage>, anyhow::Error>
        + Send
        + 'static,
{
    let room_name_ = room_name.as_bytes().to_vec();

    let res = backend_storage
        .execute_operation_with_messages::<EitherError<E>, _>(room_name_, move |versioned_game| {
            let g = interactive::InteractiveGame::new_from_state(versioned_game.game);
            let msgs = operation(&g, versioned_game.monotonic_id).map_err(EitherError::E2)?;
            Ok((
                VersionedGame {
                    room_name: versioned_game.room_name,
                    game: g.into_state(),
                    monotonic_id: versioned_game.monotonic_id,
                },
                msgs,
            ))
        })
        .await;
    match res {
        Ok(_) => true,
        Err(EitherError::E(_)) => {
            let err = GameMessage::Error(format!("Failed to {}", action_description));
            let _ = send_to_user(&user_tx, &err).await;
            false
        }
        Err(EitherError::E2(msg)) => {
            let err = GameMessage::Error(format!("Failed to {}: {}", action_description, msg));
            let _ = send_to_user(&user_tx, &err).await;
            false
        }
    }
}

async fn handle_user_action_2<S: Storage<VersionedGame, E>, E>(
    logger: Logger,
    caller: UserState,
    room_name: &str,
    name: String,
    backend_storage: S,
    msg: UserMessage,
) -> Result<(), E> {
    match msg {
        UserMessage::Beep => {
            execute_immutable_operation(
                caller.tx.clone(),
                room_name,
                backend_storage,
                move |game, _| {
                    let next_player_id = game.next_player()?;
                    let beeped_player_name = game.player_name(next_player_id)?.to_owned();
                    Ok(vec![
                        GameMessage::Message {
                            from: name,
                            message: "BEEP".to_owned(),
                        },
                        GameMessage::Beep {
                            target: beeped_player_name,
                        },
                    ])
                },
                "send appropriate beep",
            )
            .await;
        }
        UserMessage::Message(m) => {
            backend_storage
                .publish(
                    room_name.as_bytes().to_vec(),
                    GameMessage::Message {
                        from: name,
                        message: m,
                    },
                )
                .await?;
        }
        UserMessage::ReadyCheck => {
            backend_storage
                .clone()
                .publish(
                    room_name.as_bytes().to_vec(),
                    GameMessage::Message {
                        from: name.clone(),
                        message: "Is everyone ready?".to_owned(),
                    },
                )
                .await?;
            backend_storage
                .publish(
                    room_name.as_bytes().to_vec(),
                    GameMessage::ReadyCheck { from: name },
                )
                .await?;
        }
        UserMessage::Ready => {
            backend_storage
                .publish(
                    room_name.as_bytes().to_vec(),
                    GameMessage::Message {
                        from: name,
                        message: "I'm ready!".to_owned(),
                    },
                )
                .await?;
        }
        UserMessage::Kick(id) => {
            info!(logger, "Kicking user"; "other" => id.0);
            execute_operation(
                caller.tx.clone(),
                room_name,
                backend_storage,
                move |game, _| {
                    let kicked_player_name = game.player_name(id)?.to_owned();
                    game.kick(caller.player_id, id)?;
                    Ok(vec![GameMessage::Kicked {
                        target: kicked_player_name,
                    }])
                },
                "kick user",
            )
            .await;
        }
        UserMessage::Action(action) => {
            execute_operation(
                caller.tx.clone(),
                room_name,
                backend_storage,
                move |game, _| {
                    Ok(game
                        .interact(action, caller.player_id, &logger)?
                        .into_iter()
                        .map(|(data, message)| GameMessage::Broadcast { data, message })
                        .collect())
                },
                "handle user action",
            )
            .await;
        }
    }
    Ok(())
}

// async fn handle_user_action(
//     logger: &Logger,
//     caller: &UserState,
//     room: &str,
//     parent: u64,
//     games: &Games,
//     name: &str,
//     result: &[u8],
// ) -> Result<Vec<(UserState, GameMessage)>, GameMessage> {
//     let msg = serde_json::from_slice::<UserMessage>(result).map_err(|e| {
//         error!(logger, "Failed to deserialize message"; "error" => format!("{:?}", e));
//         GameMessage::Error(format!("couldn't deserialize message {:?}", e))
//     })?;
//     let mut g = games.lock().await;
//     let game = if let Some(game) = g.get_mut(room) {
//         game
//     } else {
//         error!(logger, "Game not found");
//         return Err(GameMessage::Error(format!(
//             "Couldn't find game for room {}",
//             room
//         )));
//     };
//     let logger = game.tracer(&logger, &room, Some(parent as usize));

//     let mut messages = vec![];
//     let mut broadcast_messages = vec![];
//     match msg {
//         UserMessage::Beep => {
//             let next_player_id = game
//                 .game
//                 .next_player()
//                 .map_err(|e| GameMessage::Error(e.to_string()))?;
//             for user in game.users.values() {
//                 messages.push((
//                     user.clone(),
//                     GameMessage::Message {
//                         from: name.to_owned(),
//                         message: "BEEP".to_owned(),
//                     },
//                 ));
//                 if user.player_id == next_player_id {
//                     messages.push((user.clone(), GameMessage::Beep));
//                 }
//             }
//         }
//         UserMessage::ReadyCheck => {
//             for user in game.users.values() {
//                 messages.push((
//                     user.clone(),
//                     GameMessage::Message {
//                         from: name.to_owned(),
//                         message: "Is everyone ready?".to_owned(),
//                     },
//                 ));
//                 if user.player_id != caller.player_id {
//                     messages.push((user.clone(), GameMessage::ReadyCheck));
//                 }
//             }
//         }
//         UserMessage::Ready => {
//             for user in game.users.values() {
//                 messages.push((
//                     user.clone(),
//                     GameMessage::Message {
//                         from: name.to_owned(),
//                         message: "I'm ready!".to_owned(),
//                     },
//                 ));
//             }
//         }
//         UserMessage::Message(m) => {
//             // Broadcast this msg to everyone
//             for user in game.users.values() {
//                 messages.push((
//                     user.clone(),
//                     GameMessage::Message {
//                         from: name.to_owned(),
//                         message: m.clone(),
//                     },
//                 ));
//             }
//         }
//         UserMessage::Kick(id) => {
//             info!(logger, "Kicking user"; "other" => id.0);
//             let msgs = game
//                 .game
//                 .kick(id)
//                 .map_err(|e| GameMessage::Error(e.to_string()))?;

//             for user in game.users.values() {
//                 if user.player_id == id {
//                     messages.push((user.clone(), GameMessage::Kicked));
//                 } else if let Ok(state) = game.game.dump_state_for_player(user.player_id) {
//                     messages.push((user.clone(), GameMessage::State { state }));
//                 }
//             }
//             game.users.retain(|_, u| u.player_id != id);
//             broadcast_messages.extend(msgs);
//         }
//         UserMessage::Action(m) => {
//             let msgs = game
//                 .game
//                 .interact(m, caller.player_id, &logger)
//                 .map_err(|e| GameMessage::Error(e.to_string()))?;

//             // send the updated game state to everyone!
//             for user in game.users.values() {
//                 if let Ok(state) = game.game.dump_state_for_player(user.player_id) {
//                     user.send(&GameMessage::State { state }).await;
//                 }
//             }
//             broadcast_messages.extend(msgs);
//         }
//     }

//     for user in game.users.values() {
//         messages.extend(broadcast_messages.iter().map(|(d, m)| {
//             (
//                 user.clone(),
//                 GameMessage::Broadcast {
//                     data: d.clone(),
//                     message: m.clone(),
//                 },
//             )
//         }));
//     }

//     Ok(messages)
// }

async fn user_disconnected<S: Storage<VersionedGame, E>, E>(
    room: String,
    ws_id: usize,
    _backend_storage: S,
    logger: slog::Logger,
    parent: u64,
) {
    // Stream closed up, so remove from the user list
    // let mut g = games.lock().await;
    // if let Some(game) = g.get_mut(&room) {
    //     game.users.remove(&ws_id);
    //     // If there is nobody connected anymore, drop the game entirely.
    //     if game.users.is_empty() {
    //         info!(game.tracer(&logger, &room, Some(parent)), "Removing empty room"; "room" => room.clone());
    //         g.remove(&room);
    //     }
    // }
    // drop(g);
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
