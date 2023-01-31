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
) -> bool
where
    S: Storage<VersionedGame, E>,
    E: Send,
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
        Ok(_) => true,
        Err(EitherError::E(_)) => {
            let err = GameMessage::Error(format!("Failed to {action_description}"));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            false
        }
        Err(EitherError::E2(msg)) => {
            let err = GameMessage::Error(format!("Failed to {action_description}: {msg}"));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            false
        }
    }
}

pub async fn execute_operation<S, E, F>(
    ws_id: usize,
    room_name: &str,
    backend_storage: S,
    operation: F,
    action_description: &'static str,
) -> bool
where
    S: Storage<VersionedGame, E>,
    E: Send,
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
            let err = GameMessage::Error(format!("Failed to {action_description}"));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            false
        }
        Err(EitherError::E2(msg)) => {
            let err = GameMessage::Error(format!("Failed to {action_description}: {msg}"));
            let _ = backend_storage
                .publish_to_single_subscriber(room_name_, ws_id, err)
                .await;
            false
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
