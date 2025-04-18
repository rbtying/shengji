use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use redis::{aio::ConnectionManager, AsyncCommands, RedisError};
use slog::{info, Logger};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};

use crate::storage::{State, Storage};

#[allow(clippy::type_complexity)]
pub struct RedisStorage<S: State> {
    logger: Logger,
    connection_manager: ConnectionManager,
    subscribers: Arc<Mutex<HashMap<Vec<u8>, HashMap<usize, mpsc::UnboundedSender<S::Message>>>>>,
    num_games_created: Arc<Mutex<u64>>,
    _data: PhantomData<S>,
}

#[derive(Error, Debug)]
pub enum RedisStorageError {
    #[error("Redis error")]
    RedisError(#[from] RedisError),
    #[error("Serialization error")]
    SerDeError(#[from] serde_json::Error),
    #[error("Race detected")]
    RaceDetected,
    #[error("Failed to publish message")]
    PublishError,
}

impl<S: State> RedisStorage<S> {
    pub async fn new(logger: Logger, client: redis::Client) -> Result<Self, RedisStorageError> {
        let connection_manager = client.get_tokio_connection_manager().await?;
        Ok(Self {
            logger,
            connection_manager,
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            num_games_created: Arc::new(Mutex::new(0)),
            _data: PhantomData,
        })
    }

    pub fn game_key(key: &[u8]) -> Vec<u8> {
        let mut full_key = vec![0u8; key.len() + 5];
        full_key[0..5].copy_from_slice(b"game-");
        full_key[5..].copy_from_slice(key);

        full_key
    }

    pub fn from_game_key(key: &[u8]) -> Vec<u8> {
        key[5..].to_vec()
    }

    async fn get(
        key: Vec<u8>,
        connection_manager: &mut ConnectionManager,
    ) -> Result<S, RedisStorageError> {
        let value: Option<Vec<u8>> = connection_manager.get(Self::game_key(&key)).await?;
        match value {
            Some(data) => Ok(serde_json::from_slice(&data)?),
            None => Ok(S::new_from_key(key)),
        }
    }

    async fn put(
        state: S,
        connection_manager: &mut ConnectionManager,
    ) -> Result<(), RedisStorageError> {
        let as_json = serde_json::to_vec(&state)?;
        let key = Self::game_key(state.key());
        if state.version() == 1 {
            redis::pipe()
                .atomic()
                .cmd("SET")
                .arg(key)
                .arg(as_json)
                .ignore()
                .cmd("INCR")
                .arg("states_created")
                .ignore()
                .query_async::<_, ()>(connection_manager)
                .await?;
        } else {
            redis::pipe()
                .atomic()
                .cmd("SET")
                .arg(key)
                .arg(as_json)
                .ignore()
                .query_async::<_, ()>(connection_manager)
                .await?;
        }
        Ok(())
    }

    pub async fn set_db(&mut self, db: usize) -> Result<(), RedisStorageError> {
        Ok(redis::cmd("SELECT")
            .arg(db)
            .query_async(&mut self.connection_manager)
            .await?)
    }

    pub async fn clear_all_keys(&mut self) -> Result<(), RedisStorageError> {
        Ok(redis::cmd("FLUSHDB")
            .query_async(&mut self.connection_manager)
            .await?)
    }

    fn publish(
        s: &mut HashMap<Vec<u8>, HashMap<usize, mpsc::UnboundedSender<S::Message>>>,
        key: &[u8],
        message: S::Message,
    ) {
        if let Some(subscribers) = s.get_mut(key) {
            let mut send_failed = false;
            for (_, subscriber) in subscribers.iter_mut() {
                if subscriber.send(message.clone()).is_err() {
                    send_failed |= true;
                }
            }
            if send_failed {
                subscribers.retain(|_, subscriber| !subscriber.is_closed());
            }
            if subscribers.is_empty() {
                s.remove(key);
            }
        }
    }

    async fn while_watching<R, E: From<RedisStorageError>, Fut: Future<Output = Result<R, E>>>(
        key: Vec<u8>,
        mut connection_manager: ConnectionManager,
        op: Fut,
    ) -> Result<R, E> {
        redis::cmd("WATCH")
            .arg(key.clone())
            .query_async::<_, ()>(&mut connection_manager)
            .await
            .map_err(RedisStorageError::from)?;
        let res = op.await;
        if res.is_err() {
            redis::cmd("UNWATCH")
                .arg(key)
                .query_async::<_, ()>(&mut connection_manager)
                .await
                .map_err(RedisStorageError::from)?;
        }
        res
    }
}

impl<S: State> Clone for RedisStorage<S> {
    fn clone(&self) -> Self {
        Self {
            logger: self.logger.clone(),
            connection_manager: self.connection_manager.clone(),
            subscribers: Arc::clone(&self.subscribers),
            num_games_created: Arc::clone(&self.num_games_created),
            _data: PhantomData,
        }
    }
}

#[async_trait]
impl<S: State> Storage<S, RedisStorageError> for RedisStorage<S> {
    async fn put(mut self, state: S) -> Result<(), RedisStorageError> {
        Ok(Self::put(state, &mut self.connection_manager).await?)
    }

    async fn put_cas(mut self, expected_version: u64, state: S) -> Result<(), RedisStorageError> {
        if expected_version == state.version() {
            return Ok(());
        }

        let key = state.key().to_vec();
        let mut connection_manager = self.connection_manager.clone();

        Ok(
            Self::while_watching(Self::game_key(&key), self.connection_manager, async move {
                let old_s = Self::get(key.clone(), &mut connection_manager).await?;
                if expected_version == old_s.version() {
                    Self::put(state, &mut connection_manager).await?;
                    Ok(())
                } else {
                    Err(RedisStorageError::RaceDetected)
                }
            })
            .await?,
        )
    }

    async fn get(mut self, key: Vec<u8>) -> Result<S, RedisStorageError> {
        Ok(Self::get(key, &mut self.connection_manager).await?)
    }

    async fn execute_operation_with_messages<E2, F>(
        self,
        key: Vec<u8>,
        operation: F,
    ) -> Result<u64, E2>
    where
        E2: From<RedisStorageError> + Send,
        F: FnOnce(S) -> Result<(S, Vec<S::Message>), E2> + Send + 'static,
    {
        let mut connection_manager = self.connection_manager.clone();
        Ok(Self::while_watching::<_, E2, _>(
            Self::game_key(&key),
            self.connection_manager.clone(),
            async move {
                let old_s = Self::get(key.clone(), &mut connection_manager).await?;
                let old_v = old_s.version();
                let (new_state, messages) = operation(old_s)?;
                let new_v = new_state.version();
                if new_v != old_v {
                    Self::put(new_state, &mut connection_manager).await?;
                }
                let mut s = self.subscribers.lock().await;
                for m in messages {
                    Self::publish(&mut *s, &key, m);
                }
                Ok(new_v)
            },
        )
        .await?)
    }

    async fn subscribe(
        self,
        key: Vec<u8>,
        subscriber_id: usize,
    ) -> Result<mpsc::UnboundedReceiver<S::Message>, RedisStorageError> {
        info!(self.logger, "Subscribing listener"; "key" => stringify(&key), "subscriber_id" => subscriber_id);
        let mut s = self.subscribers.lock().await;
        let (tx, rx) = mpsc::unbounded_channel();
        let ss = s.entry(key).or_default();
        ss.insert(subscriber_id, tx);
        Ok(rx)
    }

    async fn publish(self, key: Vec<u8>, message: S::Message) -> Result<(), RedisStorageError> {
        let mut s = self.subscribers.lock().await;
        Self::publish(&mut *s, &key, message);
        Ok(())
    }

    async fn publish_to_single_subscriber(
        self,
        key: Vec<u8>,
        subscriber_id: usize,
        message: S::Message,
    ) -> Result<(), RedisStorageError> {
        let s = self.subscribers.lock().await;
        if let Some(sender) = s.get(&key).and_then(|ss| ss.get(&subscriber_id)) {
            sender
                .send(message)
                .map(|_| ())
                .map_err(|_| RedisStorageError::PublishError)
        } else {
            Err(RedisStorageError::PublishError)
        }
    }

    async fn unsubscribe(mut self, key: Vec<u8>, subscriber_id: usize) {
        info!(self.logger, "Unsubscribing listener"; "key" => stringify(&key), "subscriber_id" => subscriber_id);
        let mut s = self.subscribers.lock().await;
        let should_cleanup_key = if let Some(ss) = s.get_mut(&key) {
            if ss.contains_key(&subscriber_id) {
                ss.remove(&subscriber_id);
            }
            ss.is_empty()
        } else {
            false
        };
        if should_cleanup_key {
            info!(self.logger, "Cleaning up state"; "key" => stringify(&key), "subscriber_id" => subscriber_id);
            s.remove(&key);
            let _: Result<Vec<u8>, _> = self.connection_manager.del(key).await;
        }
    }

    async fn get_all_keys(mut self) -> Result<Vec<Vec<u8>>, RedisStorageError> {
        Ok(self
            .connection_manager
            .keys::<_, Vec<Vec<u8>>>(b"game-*")
            .await?
            .iter()
            .map(|v| Self::from_game_key(v))
            .collect())
    }

    async fn get_states_created(mut self) -> Result<u64, RedisStorageError> {
        Ok(self.connection_manager.get("states_created").await?)
    }

    #[allow(clippy::if_same_then_else)]
    async fn prune(self) {
        // We walk through the key-space and remove any states which are
        // not updated in at least 2 hours.
        // We also remove any subscribers which have disconnected, and
        // subscribers for whom the game is no longer connected.
        // let mut m = self.state_map.lock().await;
        // let mut s = self.subscribers.lock().await;
        // let mut to_prune = vec![];
        // for (k, (_, t)) in m.iter() {
        //     if t.elapsed() > Duration::from_secs(2 * 3600) {
        //         to_prune.push(k.to_vec());
        //     } else if s.get(k).map(|ss| ss.is_empty()).unwrap_or(true)
        //         && t.elapsed() > Duration::from_secs(3600)
        //     {
        //         to_prune.push(k.to_vec());
        //     }
        // }
        // for k in &to_prune {
        //     m.remove(k);
        //     s.remove(k);
        // }
        // debug!(self.logger, "Ending prune"; "num_states_pruned" => to_prune.len());
    }

    async fn stats(self) -> Result<(usize, usize), RedisStorageError> {
        let self_ = self.clone();
        let num_keys = self_.get_all_keys().await?.len();
        let s = self.subscribers.lock().await;
        Ok((num_keys, s.values().map(|v| v.len()).sum()))
    }
}

fn stringify(str_like: &[u8]) -> &str {
    std::str::from_utf8(str_like).unwrap_or("not utf-8")
}
