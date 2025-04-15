use std::sync::Arc;

use slog::{debug, error, info, o, Logger};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::{sleep, Duration};

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

    // Spawn the subscription task *before* attempting registration.
    // Use Option<PlayerID> in the channel.
    let (subscribe_player_id_tx, subscribe_player_id_rx) = oneshot::channel::<Option<PlayerID>>();
    tokio::task::spawn(player_subscribe_task(
        logger.clone(),
        name.clone(),
        tx.clone(), // Clone tx for the task
        subscribe_player_id_rx,
        subscription,
    ));

    let registration_result = register_user(
        logger.clone(),
        name.clone(),
        ws_id,
        room.clone(),
        backend_storage.clone(),
        stats.clone(),
    )
    .await;

    let (player_id, join_span) = match registration_result {
        Ok(result) => result,
        Err(e) => {
            error!(logger, "User registration failed (error sent to client)"; "error" => format!("{:?}", e));
            let _ = subscribe_player_id_tx.send(None);
            sleep(Duration::from_secs(2)).await;
            return Ok(());
        }
    };

    let logger = logger.new(o!("player_id" => player_id.0));
    info!(logger, "Successfully registered user");
    let _ = subscribe_player_id_tx.send(Some(player_id));

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
    subscribe_player_id_rx: oneshot::Receiver<Option<PlayerID>>,
    mut subscription: mpsc::UnboundedReceiver<GameMessage>,
) {
    debug!(logger_, "Subscribed to messages");
    if let Ok(player_id_option) = subscribe_player_id_rx.await {
        let logger_ = logger_.new(o!("player_id" => format!("{:?}", player_id_option.map(|p|p.0))));
        debug!(logger_, "Received player ID option");
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
                // Only filter state if we have a valid player ID
                // Match on player_id_option first, then check if 'v' is GameMessage::State
                match player_id_option {
                    Some(player_id) => {
                        if let GameMessage::State { state } = v {
                            let g = InteractiveGame::new_from_state(state);
                            g.dump_state_for_player(player_id)
                                .ok()
                                .map(|filtered_state| GameMessage::State { state: filtered_state })
                        } else {
                            Some(v)
                        }
                    }
                    None => Some(v),
                }
            } else {
                None
            };

            if let Some(v_to_send) = v {
                if send_to_user(&tx, &v_to_send).await.is_err() {
                    break;
                }
            }
        }
    } else {
        error!(logger_, "Failed to receive player ID option from oneshot channel");
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
) -> Result<(PlayerID, u64), anyhow::Error> {
    let (player_id_tx, player_id_rx) = oneshot::channel();
    let logger_ = logger.clone();
    let name_ = name.clone();
    let exec_result = execute_operation(
        ws_id,
        &room,
        backend_storage.clone(),
        move |g, version, associated_websockets| {
            let (assigned_player_id, register_msgs) = g.register(name_)?;
            info!(logger_, "Joining room"; "player_id" => assigned_player_id.0);
            let mut clients_to_disconnect = vec![];
            let clients = associated_websockets.entry(assigned_player_id).or_default();
            // If the same user joined before, remove the previous entries
            // from the state-store.
            if !g.allows_multiple_sessions_per_user() {
                std::mem::swap(&mut clients_to_disconnect, clients);
            }
            clients.push(ws_id);

            player_id_tx
                .send((assigned_player_id, version, clients_to_disconnect))
                .map_err(|_| anyhow::anyhow!("Receiver dropped before player ID could be sent"))?;
            Ok(register_msgs
                .into_iter()
                .map(|(data, message)| GameMessage::Broadcast { data, message })
                .collect())
        },
        "register game",
    )
    .await;

    if let Err(e) = exec_result {
        return Err(e);
    }

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

    match player_id_rx.await {
        Ok((player_id, version, websockets_to_disconnect)) => {
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
            Ok((player_id, version))
        }
        Err(_) => {
            Err(anyhow::anyhow!("Failed to receive player ID after registration operation"))
        }
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

async fn handle_user_action<S: Storage<VersionedGame, E>, E: Send + std::fmt::Debug>(
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
            if let Err(e) = execute_immutable_operation(
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
            .await {
                error!(logger, "Beep operation failed"; "error" => format!("{:?}", e));
            }
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
            if let Err(e) = execute_operation(
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
                "kick player",
            )
            .await {
                error!(logger, "Kick operation failed"; "target_id" => id.0, "error" => format!("{:?}", e));
            }
        }
        UserMessage::Action(action) => {
            let action_clone_for_log = action.clone();
            let logger_clone_for_log = logger.clone();
            if let Err(e) = execute_operation(
                ws_id,
                room_name,
                backend_storage,
                move |g, _version, _| {
                    g.interact(action, caller, &logger).map(|msgs| {
                        msgs.into_iter()
                            .map(|(data, message)| GameMessage::Broadcast { data, message })
                            .collect()
                    })
                },
                "perform action",
            )
            .await {
                error!(logger_clone_for_log, "Action execution failed"; "action" => format!("{:?}", action_clone_for_log), "error" => format!("{:?}", e));
            }
        }
    }
    Ok(())
}

async fn user_disconnected<S: Storage<VersionedGame, E>, E: Send + std::fmt::Debug>(
    room: String,
    ws_id: usize,
    backend_storage: S,
    logger: slog::Logger,
    parent: u64,
) {
    let room_name = room.as_bytes().to_vec();
    let room_name_str = String::from_utf8_lossy(&room_name);

    info!(logger, "User disconnected, cleaning up websocket association");

    // Clean up websocket association
    if let Err(e) = execute_operation(
        ws_id,
        &room_name_str,
        backend_storage.clone(),
        move |_g, _, associated_websockets| {
            for player_websockets in associated_websockets.values_mut() {
                player_websockets.retain(|id| *id != ws_id);
            }
            associated_websockets.retain(|_, player_websockets| !player_websockets.is_empty());
            Ok(vec![])
        },
        "clean up disconnected websocket",
    )
    .await
    {
        error!(logger, "Failed to clean up websocket association on disconnect"; "error" => format!("{:?}", e));
    }

    let _ = backend_storage
        .unsubscribe(room_name, ws_id)
        .await;

    info!(logger, "Finished user disconnected cleanup"; "parent_id" => parent);
}
