use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use slog::{error, info, o, Logger};

use shengji_core::game_state::GameState;
use shengji_types::GameMessage;
use storage::{HashMapStorage, Storage};

use crate::{
    serving_types::VersionedGame,
    utils::{try_read_file, try_read_file_opt, write_state_to_disk},
    DUMP_PATH, MESSAGE_PATH, ROOT_LOGGER,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InMemoryStats {
    num_games_created: usize,
    header_messages: Vec<String>,
}

impl InMemoryStats {
    pub fn header_messages(&self) -> &[String] {
        &self.header_messages
    }
}

pub async fn load_dump_file<S: Storage<VersionedGame, E>, E: Send + std::fmt::Debug>(
    logger: Logger,
    backend_storage: S,
) -> Result<usize, anyhow::Error> {
    let mut num_games_loaded = 0usize;

    let dump = try_read_file_opt::<HashMap<String, serde_json::Value>>(&DUMP_PATH).await?;
    let dump = match dump {
        Some(dump) => dump,
        None => return Ok(0),
    };

    let futures = dump.into_iter().map(|(room_name, v)| {
        serde_json::from_value(v).map(|game| {
            backend_storage.clone().put(VersionedGame {
                room_name: room_name.as_bytes().to_vec(),
                game,
                associated_websockets: HashMap::new(),
                monotonic_id: 1,
            })
        })
    });

    for f in futures {
        if let Ok(()) = f?.await {
            num_games_loaded += 1;
        } else {
            error!(logger, "Failed to upsert initial game state");
        }
    }

    Ok(num_games_loaded)
}

pub async fn load_state(
) -> Result<(HashMapStorage<VersionedGame>, Arc<Mutex<InMemoryStats>>), anyhow::Error> {
    let backend_storage = HashMapStorage::new(ROOT_LOGGER.new(o!("component" => "storage")));

    let init_logger = ROOT_LOGGER.new(o!("dump_path" => &*DUMP_PATH));
    let ctrlc_logger = init_logger.clone();

    ctrlc::set_handler(move || {
        info!(ctrlc_logger, "Received SIGTERM, shutting down");
        std::process::exit(0);
    })
    .unwrap();

    match load_dump_file(init_logger.clone(), backend_storage.clone()).await {
        Ok(n) => {
            info!(init_logger, "Loaded games from state dump"; "num_games" => n);
        }
        Err(e) => {
            error!(init_logger, "failed to load games from disk {:?}", e);
        }
    };

    let stats = Arc::new(Mutex::new(InMemoryStats::default()));

    match try_read_file_opt::<Vec<String>>(&MESSAGE_PATH).await {
        Ok(Some(messages)) => {
            let mut stats = stats.lock().await;
            stats.header_messages = messages;
        }
        Ok(None) => (),
        Err(e) => {
            error!(init_logger, "Failed to open message file"; "error" => format!("{:?}", e));
        }
    }

    Ok((backend_storage, stats))
}

pub async fn dump_state<S: Storage<VersionedGame, E>, E>(
    backend_storage: S,
    stats: Arc<Mutex<InMemoryStats>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut state_dump: HashMap<String, GameState> = HashMap::new();

    let header_messages = try_read_file::<Vec<String>>(&MESSAGE_PATH)
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

    backend_storage.clone().prune().await;

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
    match write_state_to_disk(&*DUMP_PATH, &state_dump).await {
        Ok(()) => {
            info!(logger, "Dumped state to disk");
        }
        Err(e) => {
            error!(logger, "Failed to dump state to disk"; "error" => format!("{:?}", e));
        }
    }

    Ok(warp::reply::json(&state_dump))
}
