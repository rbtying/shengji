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
    num_games_created: u64,
    num_active_games: usize,
    num_players_online_now: usize,
    sha: &'a str,
}

#[derive(Serialize, Deserialize, Clone)]
struct VersionedGame {
    room_name: Vec<u8>,
    game: shengji_core::game_state::GameState,
    associated_websockets: HashMap<types::PlayerID, Vec<usize>>,
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
            associated_websockets: HashMap::new(),
            monotonic_id: 0,
        }
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
    let backend_storage = HashMapStorage::new(ROOT_LOGGER.new(o!("component" => "storage")));
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
                                associated_websockets: HashMap::new(),
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
        .and_then(|(backend_storage, _)| get_stats(backend_storage));

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
    backend_storage: S,
    stats: Arc<Mutex<InMemoryStats>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut state_dump: HashMap<String, game_state::GameState> = HashMap::new();

    let header_messages = try_read_file::<Vec<String>>(&*MESSAGE_PATH)
        .await
        .unwrap_or_default();
    let send_header_messages = {
        let mut stats = stats.lock().await;
        if stats.header_messages != header_messages {
            stats.header_messages = header_messages.clone();
            true
        } else {
            false
        }
    };

    let _ = backend_storage.clone().prune().await;

    let (num_games, num_players_online_now) = backend_storage
        .clone()
        .stats()
        .await
        .map_err(|_| warp::reject())?;
    let keys = backend_storage
        .clone()
        .get_all_keys()
        .await
        .map_err(|_| warp::reject())?;

    let mut num_players = 0;
    let mut num_observers = 0;
    let mut num_skipped_games = 0usize;
    let mut num_processed_games = 0usize;

    for room_name in keys {
        if let Ok(versioned_game) = backend_storage.clone().get(room_name.clone()).await {
            num_players += versioned_game.game.players().len();
            num_observers += versioned_game.game.observers().len();
            if let Ok(name) = String::from_utf8(room_name.clone()) {
                state_dump.insert(name, versioned_game.game);
            }
            num_processed_games += 1;
        } else {
            num_skipped_games += 1;
        }

        if send_header_messages {
            let _ = backend_storage
                .clone()
                .publish(
                    room_name.clone(),
                    GameMessage::Header {
                        messages: header_messages.clone(),
                    },
                )
                .await;
        }
    }

    let logger = ROOT_LOGGER.new(o!(
        "dump_path" => &*DUMP_PATH,
        "num_games" => num_games,
        "num_processed_games" => num_processed_games,
        "num_skipped_games" => num_skipped_games,
        "num_players" => num_players,
        "num_observers" => num_observers,
        "num_online_players" => num_players_online_now,
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
    backend_storage: S,
) -> Result<impl warp::Reply, warp::Rejection> {
    let num_games_created = backend_storage
        .clone()
        .get_states_created()
        .await
        .map_err(|_| warp::reject())?;
    let (num_active_games, num_players_online_now) = backend_storage
        .clone()
        .stats()
        .await
        .map_err(|_| warp::reject())?;
    Ok(warp::reply::json(&GameStats {
        num_games_created,
        num_players_online_now,
        num_active_games,
        sha: &*VERSION,
    }))
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
        let logger = logger.new(o!("room" => room.clone(), "name" => name.clone()));

        let mut subscription = match backend_storage
            .clone()
            .subscribe(room.as_bytes().to_vec(), ws_id)
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

        // Subscribe to messages for the room. After this point, we should
        // no longer use tx! It's owned by the backend storage.
        let logger_ = logger.clone();
        let name_ = name.clone();
        let (subscribe_player_id_tx, subscribe_player_id_rx) =
            oneshot::channel::<types::PlayerID>();
        tokio::task::spawn(async move {
            debug!(logger_, "Subscribed to messages");
            if let Ok(player_id) = subscribe_player_id_rx.await {
                let logger_ = logger_.new(o!("player_id" => player_id.0));
                debug!(logger_, "Received player ID");
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
                    let v = if should_send {
                        if let GameMessage::State { state } = v {
                            let g = interactive::InteractiveGame::new_from_state(state);
                            g.dump_state_for_player(player_id)
                                .ok()
                                .map(|state| GameMessage::State { state })
                        } else {
                            Some(v)
                        }
                    } else {
                        None
                    };

                    if let Some(v) = v {
                        if !send_to_user(&tx, &v).await {
                            break;
                        }
                    }
                }
            }
            debug!(logger_, "Subscription task completed");
        });

        let logger_ = logger.clone();
        let (player_id_tx, player_id_rx) = oneshot::channel();
        let name_ = name.clone();
        execute_operation(
            ws_id,
            &room,
            backend_storage.clone(),
            move |g, version, associated_websockets| {
                let (assigned_player_id, register_msgs) = g.register(name_)?;
                info!(logger_, "Joining room"; "player_id" => assigned_player_id.0);
                let mut clients_to_disconnect = vec![];
                let clients = associated_websockets
                    .entry(assigned_player_id)
                    .or_insert_with(Vec::new);
                // If the same user joined before, remove the previous entries
                // from the state-store.
                if !g.allows_multiple_sessions_per_user() {
                    std::mem::swap(&mut clients_to_disconnect, clients);
                }
                clients.push(ws_id);

                player_id_tx
                    .send((assigned_player_id, version, clients_to_disconnect))
                    .map_err(|_| anyhow::anyhow!("Couldn't send player ID back".to_owned()))?;
                Ok(register_msgs
                    .into_iter()
                    .map(|(data, message)| GameMessage::Broadcast { data, message })
                    .collect())
            },
            "register game",
        )
        .await;

        let header_messages = {
            let stats = stats.lock().await;
            stats.header_messages.clone()
        };
        let _ = backend_storage
            .clone()
            .publish_to_single_subscriber(
                room.as_bytes().to_vec(),
                ws_id,
                GameMessage::Header {
                    messages: header_messages,
                },
            )
            .await;

        if let Ok((player_id, join_span, websockets_to_disconnect)) = player_id_rx.await {
            let logger = logger.new(o!("player_id" => player_id.0));
            info!(logger, "Successfully registered user");
            let _ = subscribe_player_id_tx.send(player_id);

            for id in websockets_to_disconnect {
                info!(logger, "Disconnnecting existing client"; "kicked_ws_id" => ws_id);
                let _ = backend_storage
                    .clone()
                    .publish_to_single_subscriber(
                        room.as_bytes().to_vec(),
                        id,
                        GameMessage::Kicked {
                            target: name.clone(),
                        },
                    )
                    .await;
            }

            // Handle the main game loop
            while let Some(result) = user_ws_rx.next().await {
                let result = match result {
                    Ok(r) => r,
                    Err(e) => {
                        error!(logger, "Failed to fetch message"; "error" => format!("{:?}", e));
                        break;
                    }
                };
                if result.is_close() {
                    break;
                }
                match serde_json::from_slice::<UserMessage>(result.as_bytes()) {
                    Ok(msg) => {
                        if let Err(e) = handle_user_action(
                            logger.clone(),
                            ws_id,
                            player_id,
                            &room,
                            name.clone(),
                            backend_storage.clone(),
                            msg,
                        )
                        .await
                        {
                            let _ = backend_storage
                                .clone()
                                .publish_to_single_subscriber(
                                    room.as_bytes().to_vec(),
                                    ws_id,
                                    GameMessage::Error(format!("Unexpected error {:?}", e)),
                                )
                                .await;
                        }
                    }
                    Err(e) => {
                        error!(logger, "Failed to deserialize message"; "error" => format!("{:?}", e));
                        let _ = backend_storage
                            .clone()
                            .publish_to_single_subscriber(
                                room.as_bytes().to_vec(),
                                ws_id,
                                GameMessage::Error(format!("couldn't deserialize message {:?}", e)),
                            )
                            .await;
                    }
                }
            }

            // user_ws_rx stream will keep processing as long as the user stays
            // connected. Once they disconnect, then...
            user_disconnected(room, ws_id, backend_storage, logger, join_span).await;
        }
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
    ws_id: usize,
    room_name: &str,
    backend_storage: S,
    operation: F,
    action_description: &'static str,
) -> bool
where
    S: Storage<VersionedGame, E>,
    F: FnOnce(
            &mut interactive::InteractiveGame,
            u64,
            &mut HashMap<types::PlayerID, Vec<usize>>,
        ) -> Result<Vec<GameMessage>, anyhow::Error>
        + Send
        + 'static,
{
    let room_name_ = room_name.as_bytes().to_vec();

    let res = backend_storage
        .clone()
        .execute_operation_with_messages::<EitherError<E>, _>(
            room_name_.clone(),
            move |versioned_game| {
                let mut g = interactive::InteractiveGame::new_from_state(versioned_game.game);
                let mut associated_websockets = versioned_game.associated_websockets;
                let mut msgs = operation(
                    &mut g,
                    versioned_game.monotonic_id,
                    &mut associated_websockets,
                )
                .map_err(EitherError::E2)?;
                let game = g.into_state();
                msgs.push(GameMessage::State {
                    state: game.clone(),
                });
                Ok((
                    VersionedGame {
                        room_name: versioned_game.room_name,
                        game,
                        associated_websockets,
                        monotonic_id: versioned_game.monotonic_id + 1,
                    },
                    msgs,
                ))
            },
        )
        .await;
    match res {
        Ok(_) => true,
        Err(EitherError::E(_)) => {
            let err = GameMessage::Error(format!("Failed to {}", action_description));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            false
        }
        Err(EitherError::E2(msg)) => {
            let err = GameMessage::Error(format!("Failed to {}: {}", action_description, msg));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            false
        }
    }
}

async fn execute_immutable_operation<S, E, F>(
    ws_id: usize,
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
        .clone()
        .execute_operation_with_messages::<EitherError<E>, _>(
            room_name_.clone(),
            move |versioned_game| {
                let g = interactive::InteractiveGame::new_from_state(versioned_game.game);
                let msgs = operation(&g, versioned_game.monotonic_id).map_err(EitherError::E2)?;
                Ok((
                    VersionedGame {
                        game: g.into_state(),
                        room_name: versioned_game.room_name,
                        monotonic_id: versioned_game.monotonic_id,
                        associated_websockets: versioned_game.associated_websockets,
                    },
                    msgs,
                ))
            },
        )
        .await;
    match res {
        Ok(_) => true,
        Err(EitherError::E(_)) => {
            let err = GameMessage::Error(format!("Failed to {}", action_description));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            false
        }
        Err(EitherError::E2(msg)) => {
            let err = GameMessage::Error(format!("Failed to {}: {}", action_description, msg));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            false
        }
    }
}

async fn handle_user_action<S: Storage<VersionedGame, E>, E>(
    logger: Logger,
    ws_id: usize,
    caller: types::PlayerID,
    room_name: &str,
    name: String,
    backend_storage: S,
    msg: UserMessage,
) -> Result<(), E> {
    match msg {
        UserMessage::Beep => {
            execute_immutable_operation(
                ws_id,
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
                ws_id,
                room_name,
                backend_storage,
                move |game, _, _| {
                    let kicked_player_name = game.player_name(id)?.to_owned();
                    game.kick(caller, id)?;
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
                ws_id,
                room_name,
                backend_storage,
                move |game, _, _| {
                    Ok(game
                        .interact(action, caller, &logger)?
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

async fn user_disconnected<S: Storage<VersionedGame, E>, E>(
    room: String,
    ws_id: usize,
    backend_storage: S,
    logger: slog::Logger,
    parent: u64,
) {
    execute_operation(
        ws_id,
        &room,
        backend_storage.clone(),
        move |_, _, associated_websockets| {
            for ws in associated_websockets.values_mut() {
                ws.retain(|w| *w != ws_id);
            }
            Ok(vec![])
        },
        "disconnect player",
    )
    .await;
    let _ = backend_storage
        .unsubscribe(room.as_bytes().to_vec(), ws_id)
        .await;
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
