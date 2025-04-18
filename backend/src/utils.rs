use std::collections::HashMap;
use std::io::{self, ErrorKind};

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use shengji_core::interactive::InteractiveGame;
use shengji_mechanics::types::PlayerID;
use shengji_types::GameMessage;
use storage::Storage;

use crate::serving_types::VersionedGame;

pub async fn try_read_file<M: serde::de::DeserializeOwned>(path: &'_ str) -> Result<M, io::Error> {
    let mut f = tokio::fs::File::open(path).await?;
    let mut data = vec![];
    f.read_to_end(&mut data).await?;
    Ok(serde_json::from_slice(&data)?)
}

pub async fn try_read_file_opt<M: serde::de::DeserializeOwned>(
    path: &'_ str,
) -> Result<Option<M>, io::Error> {
    match try_read_file(path).await {
        Ok(t) => Ok(Some(t)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

pub async fn write_state_to_disk<M: serde::ser::Serialize>(
    path: &'_ str,
    state: &HashMap<String, M>,
) -> std::io::Result<()> {
    let mut f = tokio::fs::File::create(path).await?;
    let json = serde_json::to_vec(state)?;
    f.write_all(&json).await?;
    f.sync_all().await?;

    Ok(())
}

pub async fn execute_immutable_operation<S, E, F>(
    ws_id: usize,
    room_name: &str,
    backend_storage: S,
    operation: F,
    action_description: &'static str,
) -> Result<(), anyhow::Error>
where
    S: Storage<VersionedGame, E>,
    E: Send + std::fmt::Debug,
    F: FnOnce(&InteractiveGame, u64) -> Result<Vec<GameMessage>, anyhow::Error> + Send + 'static,
{
    let room_name_ = room_name.as_bytes().to_vec();

    let res = backend_storage
        .clone()
        .execute_operation_with_messages::<EitherError<E>, _>(
            room_name_.clone(),
            move |versioned_game| {
                let g = InteractiveGame::new_from_state(versioned_game.game);
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
        Ok(_) => Ok(()),
        Err(EitherError::E(e)) => {
            let err_msg = format!("Failed to {action_description} due to storage error");
            let err = GameMessage::Error(err_msg.clone());
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            Err(anyhow::anyhow!("{}: {:?}", err_msg, e))
        }
        Err(EitherError::E2(msg)) => {
            let err = GameMessage::Error(format!("Failed to {action_description}: {msg}"));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            Err(msg)
        }
    }
}

pub async fn execute_operation<S, E, F>(
    ws_id: usize,
    room_name: &str,
    backend_storage: S,
    operation: F,
    action_description: &'static str,
) -> Result<(), anyhow::Error>
where
    S: Storage<VersionedGame, E>,
    E: Send + std::fmt::Debug,
    F: FnOnce(
            &mut InteractiveGame,
            u64,
            &mut HashMap<PlayerID, Vec<usize>>,
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
                let mut g = InteractiveGame::new_from_state(versioned_game.game);
                let mut associated_websockets = versioned_game.associated_websockets;
                let msgs_to_broadcast = operation(
                    &mut g,
                    versioned_game.monotonic_id,
                    &mut associated_websockets,
                )
                .map_err(EitherError::E2)?;
                let final_game_state = g.into_state();
                Ok((
                    VersionedGame {
                        room_name: versioned_game.room_name,
                        game: final_game_state,
                        associated_websockets,
                        monotonic_id: versioned_game.monotonic_id + 1,
                    },
                    msgs_to_broadcast,
                ))
            },
        )
        .await;

    match res {
        Ok(new_version) => match backend_storage.clone().get(room_name_.clone()).await {
            Ok(updated_versioned_game) => {
                if updated_versioned_game.monotonic_id == new_version {
                    let targeted_state_msg = GameMessage::State {
                        state: updated_versioned_game.game,
                    };
                    let _ = backend_storage
                        .publish_to_single_subscriber(room_name_, ws_id, targeted_state_msg)
                        .await;
                    Ok(())
                } else {
                    let err_msg = format!("Operation succeeded but version mismatch after fetching state (expected {}, got {})", new_version, updated_versioned_game.monotonic_id);
                    let err = GameMessage::Error(err_msg.clone());
                    let _ = backend_storage
                        .publish_to_single_subscriber(room_name_.clone(), ws_id, err)
                        .await;
                    Err(anyhow::anyhow!(err_msg))
                }
            }
            Err(e) => {
                let err_msg = format!("Operation succeeded but failed to fetch updated state");
                let err = GameMessage::Error(err_msg.clone());
                let _ = backend_storage
                    .publish_to_single_subscriber(room_name_.clone(), ws_id, err)
                    .await;
                Err(anyhow::anyhow!("{}: {:?}", err_msg, e))
            }
        },
        Err(EitherError::E(e)) => {
            let err_msg = format!("Failed to {action_description} due to storage error");
            let err = GameMessage::Error(err_msg.clone());
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            Err(anyhow::anyhow!("{}: {:?}", err_msg, e))
        }
        Err(EitherError::E2(msg)) => {
            let err = GameMessage::Error(format!("Failed to {action_description}: {msg}"));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            Err(msg)
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
