#![deny(warnings)]

use std::env;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use axum::{
    extract::ws::{Message, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Extension, Json, Router,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use slog::{debug, error, info, o, Drain, Logger};
use tokio::sync::{mpsc, Mutex};

#[cfg(feature = "dynamic")]
use axum::routing::get_service;
#[cfg(not(feature = "dynamic"))]
use axum::{
    body::{Empty, Full},
    extract::Path,
    response::Response,
};
#[cfg(feature = "dynamic")]
use tower_http::services::ServeDir;

use shengji_core::settings;
use shengji_mechanics::types::FULL_DECK;
use shengji_types::ZSTD_ZSTD_DICT;
use storage::{HashMapStorage, Storage};

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
    static ref WEBSOCKET_HOST: Option<String> = {
        std::env::var("WEBSOCKET_HOST").ok()
    };
}

async fn runtime_settings() -> impl IntoResponse {
    let body = match WEBSOCKET_HOST.as_ref() {
        Some(s) => format!(
            "window._WEBSOCKET_HOST = \"{}\";window._VERSION = \"{}\";",
            s, *VERSION,
        ),
        None => format!(
            "window._WEBSOCKET_HOST = null;window._VERSION = \"{}\";",
            *VERSION
        ),
    };
    (
        [(http::header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        body,
    )
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    ctrlc::set_handler(move || {
        info!(ROOT_LOGGER, "Received SIGTERM, shutting down");
        std::process::exit(0);
    })
    .unwrap();

    let (backend_storage, stats) = state_dump::load_state().await?;

    tokio::task::spawn(periodically_dump_state(
        backend_storage.clone(),
        stats.clone(),
    ));

    let app = Router::new()
        .route("/api", get(handle_websocket))
        .route(
            "/default_settings.json",
            get(|| async { Json(settings::PropagatedState::default()) }),
        )
        .route("/full_state.json", get(state_dump::dump_state))
        .route("/stats", get(get_stats))
        .route("/runtime.js", get(runtime_settings))
        .route("/cards.json", get(|| async { Json(CARDS_JSON.clone()) }))
        .route(
            "/rules",
            get(|| async { Redirect::permanent("/rules.html") }),
        );

    #[cfg(feature = "dynamic")]
    let app = app.fallback_service(
        get_service(ServeDir::new("../frontend/dist").fallback(ServeDir::new("../favicon")))
            .handle_error(handle_error),
    );
    #[cfg(not(feature = "dynamic"))]
    let app = app
        .route(
            "/",
            get(|| async { serve_static_routes(Path("index.html".to_string())).await }),
        )
        .route("/*path", get(serve_static_routes));

    let app = app
        .layer(Extension(backend_storage))
        .layer(Extension(stats));

    axum::Server::bind(&SocketAddr::from(([0, 0, 0, 0], 3030)))
        .serve(app.into_make_service())
        .await?;

    info!(ROOT_LOGGER, "Shutting down");
    Ok(())
}

#[cfg(feature = "dynamic")]
async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

#[derive(Debug, Serialize, Deserialize)]
struct GameStats {
    num_games_created: u64,
    num_active_games: usize,
    num_players_online_now: usize,
    sha: &'static str,
}

async fn get_stats(
    Extension(backend_storage): Extension<HashMapStorage<VersionedGame>>,
) -> Result<Json<GameStats>, &'static str> {
    let num_games_created = backend_storage
        .clone()
        .get_states_created()
        .await
        .map_err(|_| "failed to get number of games created")?;
    let (num_active_games, num_players_online_now) = backend_storage
        .clone()
        .stats()
        .await
        .map_err(|_| "failed to get number of active games and online players")?;
    Ok(Json(GameStats {
        num_games_created,
        num_players_online_now,
        num_active_games,
        sha: &VERSION,
    }))
}

async fn periodically_dump_state(
    backend_storage: HashMapStorage<VersionedGame>,
    stats: Arc<Mutex<InMemoryStats>>,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        let _ =
            state_dump::dump_state(Extension(backend_storage.clone()), Extension(stats.clone()))
                .await;
    }
}

async fn handle_websocket(
    ws: WebSocketUpgrade,
    Extension(backend_storage): Extension<HashMapStorage<VersionedGame>>,
    Extension(stats): Extension<Arc<Mutex<InMemoryStats>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|ws| {
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
                let _ = user_ws_tx.send(Message::Binary(v)).await;
            }
            debug!(logger_, "Ending tx task");
        });

        // And another channel to receive messages from the websocket
        let logger_ = logger.clone();
        let (tx2, rx2) = mpsc::unbounded_channel();
        tokio::task::spawn(async move {
            while let Some(result) = user_ws_rx.next().await {
                match result {
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Ok(Message::Binary(r)) => {
                        let _ = tx2.send(r);
                    }
                    Ok(Message::Text(r)) => {
                        let _ = tx2.send(r.into_bytes());
                    }
                    Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => (),
                    Err(e) => {
                        error!(logger_, "Failed to fetch message"; "error" => format!("{e:?}"));
                        break;
                    }
                }
            }
            debug!(logger_, "Ending rx task");
        });

        shengji_handler::entrypoint(tx, rx2, ws_id, logger, backend_storage, stats)
    })
}

#[cfg(not(feature = "dynamic"))]
async fn serve_static_routes(Path(path): Path<String>) -> impl IntoResponse {
    static DIST: include_dir::Dir<'_> = include_dir::include_dir!("frontend/dist");
    static FAVICON: include_dir::Dir<'_> = include_dir::include_dir!("favicon");
    let mime_type = mime_guess::from_path(&path).first_or_text_plain();

    match DIST.get_file(&path).or_else(|| FAVICON.get_file(&path)) {
        Some(f) => Response::builder()
            .status(StatusCode::OK)
            .header(
                http::header::CONTENT_TYPE,
                http::HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(axum::body::boxed(Full::from(f.contents())))
            .unwrap(),
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::boxed(Empty::new()))
            .unwrap(),
    }
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
