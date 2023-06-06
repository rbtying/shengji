use std::sync::Arc;

use slog::{debug, error, info, o, Logger};
use tokio::sync::{mpsc, oneshot, Mutex};

use shengji_core::interactive::InteractiveGame;
use shengji_mechanics::types::PlayerID;
use shengji_types::GameMessage;
use storage::Storage;

use crate::{
    serving_types::{JoinRoom, UserMessage, VersionedGame},
    state_dump::InMemoryStats,
    utils::{execute_immutable_operation, execute_operation},
    ZSTD_COMPRESSOR,
};

pub async fn entrypoint<S: Storage<VersionedGame, E>, E: std::fmt::Debug + Send>(
    tx: mpsc::UnboundedSender<Vec<u8>>,
    rx: mpsc::UnboundedReceiver<Vec<u8>>,
    ws_id: usize,
    logger: Logger,
    backend_storage: S,
    stats: Arc<Mutex<InMemoryStats>>,
) {
    let _ = handle_user_connected(tx, rx, ws_id, logger, backend_storage, stats).await;
}

async fn send_to_user(
    tx: &'_ mpsc::UnboundedSender<Vec<u8>>,
    msg: &GameMessage,
) -> Result<(), anyhow::Error> {
    if let Ok(j) = serde_json::to_vec(&msg) {
        if let Ok(s) = ZSTD_COMPRESSOR.lock().unwrap().compress(&j) {
            if tx.send(s).is_ok() {
                return Ok(());
            }
        }
    }
    Err(anyhow::anyhow!("Unable to send message to user {:?}", msg))
}

async fn handle_user_connected<S: Storage<VersionedGame, E>, E: std::fmt::Debug + Send>(
    tx: mpsc::UnboundedSender<Vec<u8>>,
    mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
    ws_id: usize,
    logger: Logger,
    backend_storage: S,
    stats: Arc<Mutex<InMemoryStats>>,
) -> Result<(), anyhow::Error> {
    let (room, name) = loop {
        if let Some(msg) = rx.recv().await {
            let err = match serde_json::from_slice(&msg) {
                Ok(JoinRoom { room_name, name }) if room_name.len() == 16 && name.len() < 32 => {
                    break (room_name, name);
                }
                Ok(_) => GameMessage::Error("invalid room or name".to_string()),
                Err(err) => GameMessage::Error(format!("couldn't deserialize message {err:?}")),
            };

            send_to_user(&tx, &err).await?;
        } else {
            Err(anyhow::anyhow!("no message on socket"))?;
        }
    };

    let logger = logger.new(o!("room" => room.clone(), "name" => name.clone()));

    let subscription = match backend_storage
        .clone()
        .subscribe(room.as_bytes().to_vec(), ws_id)
        .await
    {
        Ok(sub) => sub,
        Err(e) => {
            let _ = send_to_user(
                &tx,
                &GameMessage::Error(format!("Failed to join room: {e:?}")),
            )
            .await;
            return Err(anyhow::anyhow!("Failed to join room {:?}", e));
        }
    };

    // Subscribe to messages for the room. After this point, we should
    // no longer use tx! It's owned by the backend storage.
    let (subscribe_player_id_tx, subscribe_player_id_rx) = oneshot::channel::<PlayerID>();
    tokio::task::spawn(player_subscribe_task(
        logger.clone(),
        name.clone(),
        tx.clone(),
        subscribe_player_id_rx,
        subscription,
    ));

    let (player_id, join_span) = register_user(
        logger.clone(),
        name.clone(),
        ws_id,
        room.clone(),
        backend_storage.clone(),
        stats.clone(),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Failed to register user"))?;

    let logger = logger.new(o!("player_id" => player_id.0));
    info!(logger, "Successfully registered user");
    let _ = subscribe_player_id_tx.send(player_id);

    run_game_for_player(
        logger.clone(),
        ws_id,
        player_id,
        room.clone(),
        name,
        backend_storage.clone(),
        rx,
    )
    .await;

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(room, ws_id, backend_storage, logger, join_span).await;
    Ok(())
}

async fn player_subscribe_task(
    logger_: Logger,
    name_: String,
    tx: mpsc::UnboundedSender<Vec<u8>>,
    subscribe_player_id_rx: oneshot::Receiver<PlayerID>,
    mut subscription: mpsc::UnboundedReceiver<GameMessage>,
) {
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
                GameMessage::Beep { target } | GameMessage::Kicked { target } => *target == name_,
                GameMessage::ReadyCheck { from } => *from != name_,
            };
            let v = if should_send {
                if let GameMessage::State { state } = v {
                    let g = InteractiveGame::new_from_state(state);
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
                if send_to_user(&tx, &v).await.is_err() {
                    break;
                }
            }
        }
    }
    debug!(logger_, "Subscription task completed");
}

async fn register_user<S: Storage<VersionedGame, E>, E: std::fmt::Debug + Send>(
    logger: Logger,
    name: String,
    ws_id: usize,
    room: String,
    backend_storage: S,
    stats: Arc<Mutex<InMemoryStats>>,
) -> Result<(PlayerID, u64), ()> {
    let (player_id_tx, player_id_rx) = oneshot::channel();
    let logger_ = logger.clone();
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
        stats.header_messages().to_vec()
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

    if let Ok((player_id, ws_id, websockets_to_disconnect)) = player_id_rx.await {
        for id in websockets_to_disconnect {
            info!(logger, "Disconnnecting existing client"; "kicked_ws_id" => id);
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
        Ok((player_id, ws_id))
    } else {
        Err(())
    }
}

async fn run_game_for_player<S: Storage<VersionedGame, E>, E: Send + std::fmt::Debug>(
    logger: Logger,
    ws_id: usize,
    player_id: PlayerID,
    room: String,
    name: String,
    backend_storage: S,
    mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
) {
    debug!(logger, "Entering main game loop");
    // Handle the main game loop
    while let Some(result) = rx.recv().await {
        match serde_json::from_slice::<UserMessage>(&result) {
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
                            GameMessage::Error(format!("Unexpected error {e:?}")),
                        )
                        .await;
                }
            }
            Err(e) => {
                error!(logger, "Failed to deserialize message"; "error" => format!("{e:?}"));
                let _ = backend_storage
                    .clone()
                    .publish_to_single_subscriber(
                        room.as_bytes().to_vec(),
                        ws_id,
                        GameMessage::Error(format!("couldn't deserialize message {e:?}")),
                    )
                    .await;
            }
        }
    }
    debug!(logger, "Exiting main game loop");
}

async fn handle_user_action<S: Storage<VersionedGame, E>, E: Send>(
    logger: Logger,
    ws_id: usize,
    caller: PlayerID,
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

async fn user_disconnected<S: Storage<VersionedGame, E>, E: Send>(
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
    backend_storage
        .unsubscribe(room.as_bytes().to_vec(), ws_id)
        .await;
    info!(logger, "Websocket disconnected";
        "room" => room,
        "parent_span" => format!("{room}:{parent}"),
        "span" => format!("{room}:ws_{ws_id}")
    );
}
