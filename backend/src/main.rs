#![deny(warnings)]

use std::env;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use slog::{debug, error, info, o, Drain, Logger};
use tokio::sync::{mpsc, Mutex};
use warp::ws::{Message, WebSocket};
use warp::Filter;

use shengji_core::{settings, types::FULL_DECK};
use shengji_types::ZSTD_ZSTD_DICT;
use storage::Storage;

mod serving_types;
mod shengji_handler;
mod state_dump;
mod utils;

use serving_types::{CardsBlob, VersionedGame};
use state_dump::InMemoryStats;

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

lazy_static::lazy_static! {
    static ref CARDS_JSON: CardsBlob = CardsBlob {
        cards: FULL_DECK.iter().map(|c| c.as_info()).collect()
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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let (backend_storage, stats) = state_dump::load_state().await?;

    tokio::task::spawn(periodically_dump_state(
        backend_storage.clone(),
        stats.clone(),
    ));

    let games_filter = warp::any().map(move || (backend_storage.clone(), stats.clone()));

    let api = warp::path("api")
        .and(warp::ws())
        .and(games_filter.clone())
        .map(|ws: warp::ws::Ws, (backend_storage, stats)| {
            ws.on_upgrade(move |socket| handle_websocket(socket, backend_storage, stats))
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
        .and_then(|(backend_storage, stats)| state_dump::dump_state(backend_storage, stats));
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

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;

    info!(ROOT_LOGGER, "Shutting down");
    Ok(())
}

async fn get_stats<S: Storage<VersionedGame, E>, E>(
    backend_storage: S,
) -> Result<impl warp::Reply, warp::Rejection> {
    #[derive(Debug, Serialize, Deserialize)]
    struct GameStats<'a> {
        num_games_created: u64,
        num_active_games: usize,
        num_players_online_now: usize,
        sha: &'a str,
    }

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
        sha: &VERSION,
    }))
}

async fn default_propagated() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&settings::PropagatedState::default()))
}

async fn periodically_dump_state<S: Storage<VersionedGame, E>, E>(
    backend_storage: S,
    stats: Arc<Mutex<InMemoryStats>>,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        let _ = state_dump::dump_state(backend_storage.clone(), stats.clone()).await;
    }
}

async fn handle_websocket<S: Storage<VersionedGame, E>, E: std::fmt::Debug + Send>(
    ws: WebSocket,
    backend_storage: S,
    stats: Arc<Mutex<InMemoryStats>>,
) {
    let ws_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    let logger = ROOT_LOGGER.new(o!("ws_id" => ws_id));
    info!(logger, "Websocket connection initialized");
    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let logger_ = logger.clone();
    let (tx, mut rx) = mpsc::unbounded_channel();
    tokio::task::spawn(async move {
        while let Some(v) = rx.recv().await {
            let _ = user_ws_tx.send(Message::binary(v)).await;
        }
        debug!(logger_, "Ending tx task");
    });

    // And another channel to receive messages from the websocket
    let logger_ = logger.clone();
    let (tx2, rx2) = mpsc::unbounded_channel();
    tokio::task::spawn(async move {
        while let Some(result) = user_ws_rx.next().await {
            match result {
                Ok(r) if r.is_close() => {
                    break;
                }
                Ok(r) => {
                    let _ = tx2.send(r.into_bytes());
                }
                Err(e) => {
                    error!(logger_, "Failed to fetch message"; "error" => format!("{:?}", e));
                    break;
                }
            }
        }
        debug!(logger_, "Ending rx task");
    });

    shengji_handler::entrypoint(tx, rx2, ws_id, logger, backend_storage, stats).await
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
