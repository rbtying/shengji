#![deny(warnings)]
#![feature(const_fn)]
#![feature(const_if_match)]

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex};
use warp::ws::{Message, WebSocket};
use warp::{http::Response, Filter};

pub mod game_state;
pub mod hands;
pub mod interactive;
pub mod trick;
pub mod types;

/// Our global unique user id counter.
static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

struct GameState {
    game: interactive::InteractiveGame,
    users: HashMap<usize, UserState>,
}

struct UserState {
    player_id: types::PlayerID,
    tx: mpsc::UnboundedSender<Result<Message, warp::Error>>,
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
}

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
    Error(String),
}

#[tokio::main]
async fn main() {
    let games = Arc::new(Mutex::new(HashMap::new()));
    let games = warp::any().map(move || games.clone());

    // GET /api -> websocket upgrade
    let api = warp::path("api")
        // The `ws()` filter will prepare Websocket handshake...
        .and(warp::ws())
        .and(games)
        .map(|ws: warp::ws::Ws, games| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, games))
        });

    // GET / -> index html
    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let js = warp::path("game.js").map(|| {
        Response::builder()
            .header("Content-Type", "text/javascript")
            .body(JS)
    });
    let routes = index.or(js).or(api);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(ws: WebSocket, games: Games) {
    // Use a counter to assign a new unique ID for this user.
    let ws_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);

    eprintln!("new api user: {}", ws_id);

    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
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
                    let err = GameMessage::Error("invalid room or name".to_string());
                    if let Ok(msg) = serde_json::to_string(&err) {
                        if let Err(_) = tx.send(Ok(Message::text(msg))) {
                            break;
                        }
                    }
                }
                Err(err) => {
                    let err = GameMessage::Error(format!("couldn't deserialize message {:?}", err));
                    if let Ok(msg) = serde_json::to_string(&err) {
                        if let Err(_) = tx.send(Ok(Message::text(msg))) {
                            break;
                        }
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
            });
            let player_id = match game.game.register(name.clone()) {
                Ok(player_id) => player_id,
                Err(err) => {
                    let err = GameMessage::Error(format!("couldn't register for game {:?}", err));
                    if let Ok(msg) = serde_json::to_string(&err) {
                        let _ = tx.send(Ok(Message::text(msg)));
                    }
                    return;
                }
            };
            game.users.insert(
                ws_id,
                UserState {
                    player_id,
                    tx: tx.clone(),
                },
            );
            // send the updated game state to everyone!
            for user in game.users.values() {
                if let Ok((state, cards)) = game.game.dump_state_for_player(user.player_id) {
                    if let Ok(s) = serde_json::to_string(&GameMessage::State { state, cards }) {
                        let _ = user.tx.send(Ok(Message::text(s)));
                    }
                }
            }
            player_id
        };
        let games2 = games.clone();

        // Every time the user sends a message, broadcast it to
        // all other users...
        while let Some(result) = user_ws_rx.next().await {
            match result {
                Ok(msg) => {
                    match serde_json::from_slice::<UserMessage>(msg.as_bytes()) {
                        Ok(UserMessage::Message(m)) => {
                            // Broadcast this msg to everyone
                            let g = games.lock().await;
                            if let Some(game) = g.get(&room) {
                                for user in game.users.values() {
                                    if let Ok(s) = serde_json::to_string(&GameMessage::Message {
                                        from: name.clone(),
                                        message: m.clone(),
                                    }) {
                                        let _ = user.tx.send(Ok(Message::text(s)));
                                    }
                                }
                            }
                        }
                        Ok(UserMessage::Action(m)) => {
                            let g = games.lock().await;
                            if let Some(game) = g.get(&room) {
                                match game.game.interact(m, player_id) {
                                    Ok(()) => {
                                        // send the updated game state to everyone!
                                        for user in game.users.values() {
                                            if let Ok((state, cards)) =
                                                game.game.dump_state_for_player(user.player_id)
                                            {
                                                if let Ok(s) =
                                                    serde_json::to_string(&GameMessage::State {
                                                        state,
                                                        cards,
                                                    })
                                                {
                                                    let _ = user.tx.send(Ok(Message::text(s)));
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        // send the error back to the requester
                                        let err = GameMessage::Error(format!("{}", err));
                                        if let Ok(msg) = serde_json::to_string(&err) {
                                            if let Err(_) = tx.send(Ok(Message::text(msg))) {
                                                break;
                                            }
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
                            if let Ok(msg) = serde_json::to_string(&err) {
                                if let Err(_) = tx.send(Ok(Message::text(msg))) {
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("websocket error(uid={}): {}", ws_id, e);
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
    eprintln!("good bye user: {}, {}", room, ws_id);

    // Stream closed up, so remove from the user list
    let mut g = games.lock().await;
    if let Some(game) = g.get_mut(&room) {
        if let Some(UserState { player_id, .. }) = game.users.remove(&ws_id) {
            // If there are no other connections with this user id, try to kick the player out of
            // the room.
            if game
                .users
                .values()
                .filter(|k| k.player_id == player_id)
                .next()
                .is_none()
            {
                if let Ok(_) = game.game.kick(player_id) {
                    eprintln!("kicking all players with ID {:?} from {}", player_id, room);
                    for user in game.users.values() {
                        if let Ok((state, cards)) = game.game.dump_state_for_player(user.player_id)
                        {
                            if let Ok(s) =
                                serde_json::to_string(&GameMessage::State { state, cards })
                            {
                                let _ = user.tx.send(Ok(Message::text(s)));
                            }
                        }
                    }
                }
            }
        }
        // If there is nobody connected anymore, drop the game entirely.
        if game.users.is_empty() {
            eprintln!("deleting room {}", room);
            g.remove(&room);
        }
    }
}

static INDEX_HTML: &str = include_str!("index.html");
static JS: &str = include_str!("game.js");
