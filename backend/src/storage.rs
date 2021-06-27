use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use slog::{debug, info, Logger};
use tokio::sync::{mpsc, Mutex};

pub trait State: Serialize + DeserializeOwned + Clone + Send {
    /// Messages that can be sent by operations applied to the state.
    type Message: Serialize + DeserializeOwned + Clone + Send;

    /// The key that this state corresponds to.
    fn key(&self) -> &[u8];
    fn version(&self) -> u64;

    /// The version of the state. Changes to state require changes in the
    /// version. The default version number must be zero.
    fn new_from_key(key: Vec<u8>) -> Self;
}

#[async_trait]
pub trait Storage<S: State, E>: Clone + Send {
    /// Put the state into storage, overwriting any existing value.
    async fn put(self, state: S) -> Result<(), E>;

    /// Put the state into storage. If the version on the server doesn't match
    /// the expected version, return an error.
    async fn put_cas(self, expected_version: u64, state: S) -> Result<(), E>;

    /// Get the state corresponding to the key from storage. If it doesn't
    /// exist, a new state will be instantiated with a default version.
    async fn get(self, key: Vec<u8>) -> Result<S, E>;

    /// Execute the provided operation based off of the version of the state.
    ///
    /// If the operation succeeds, the returned messages will be published to
    /// the key and the corresponding state will be stored if its version
    /// differs from the already-stored version.
    ///
    /// If the operation fails, state will not be changed, and the error will be
    /// returned.
    ///
    /// This operation may also fail if the stored state's `version` field
    /// differs from the one which is fetched at the beginning of the operation
    /// -- it has compare-and-set semantics.
    async fn execute_operation_with_messages<E2, F>(
        self,
        key: Vec<u8>,
        operation: F,
    ) -> Result<u64, E2>
    where
        E2: From<E>,
        F: FnOnce(S) -> Result<(S, Vec<S::Message>), E2> + Send + 'static;

    /// Subscribe to messages about a given key. The `subscriber_id` is expected
    /// to be unique across all subscribers.
    async fn subscribe(
        self,
        key: Vec<u8>,
        subscriber_id: usize,
    ) -> Result<mpsc::UnboundedReceiver<S::Message>, E>;
    /// Publish to all subscribers for a given key.
    async fn publish(self, key: Vec<u8>, message: S::Message) -> Result<(), E>;
    /// Publish a message to a single subscriber, identified by subscriber id.
    async fn publish_to_single_subscriber(
        self,
        key: Vec<u8>,
        subscriber_id: usize,
        message: S::Message,
    ) -> Result<(), E>;
    /// Unsubscribe a given subscriber and remove it from tracking.
    async fn unsubscribe(self, key: Vec<u8>, subscriber_id: usize);

    /// This should be called on a regular basis to ensure that we don't leave
    /// stale state in the storage layer.
    async fn prune(self);
    /// Count the number of active subscriptions and active states.
    async fn stats(self) -> Result<(usize, usize), E>;
    /// Get all of the keys stored in this storage backend.
    async fn get_all_keys(self) -> Result<Vec<Vec<u8>>, E>;
    /// Get the number of states that have been newly created.
    async fn get_states_created(self) -> Result<u64, E>;
}

#[allow(clippy::type_complexity)]
pub struct HashMapStorage<S: State> {
    logger: Logger,
    state_map: Arc<Mutex<HashMap<Vec<u8>, (S, Instant)>>>,
    subscribers: Arc<Mutex<HashMap<Vec<u8>, HashMap<usize, mpsc::UnboundedSender<S::Message>>>>>,
    num_games_created: Arc<Mutex<u64>>,
    _data: PhantomData<S>,
}

impl<S: State> HashMapStorage<S> {
    pub fn new(logger: Logger) -> Self {
        Self {
            logger,
            state_map: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            num_games_created: Arc::new(Mutex::new(0)),
            _data: PhantomData,
        }
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
}

impl<S: State> Clone for HashMapStorage<S> {
    fn clone(&self) -> Self {
        Self {
            logger: self.logger.clone(),
            state_map: Arc::clone(&self.state_map),
            subscribers: Arc::clone(&self.subscribers),
            num_games_created: Arc::clone(&self.num_games_created),
            _data: PhantomData,
        }
    }
}

#[async_trait]
impl<S: State> Storage<S, ()> for HashMapStorage<S> {
    async fn put(self, state: S) -> Result<(), ()> {
        let mut m = self.state_map.lock().await;
        if !m.contains_key(state.key()) {
            *self.num_games_created.lock().await += 1;
            info!(self.logger, "Initializing state"; "key" => stringify(state.key()));
        }
        m.insert(state.key().to_vec(), (state, Instant::now()));
        Ok(())
    }

    async fn put_cas(self, expected_version: u64, state: S) -> Result<(), ()> {
        let mut m = self.state_map.lock().await;
        if !m.contains_key(state.key()) {
            *self.num_games_created.lock().await += 1;
            info!(self.logger, "Initializing state"; "key" => stringify(state.key()));
        }
        if m.get(state.key()).map(|s| s.0.version()).unwrap_or(0) == expected_version {
            if state.version() != expected_version {
                m.insert(state.key().to_vec(), (state, Instant::now()));
                if expected_version == 0 {
                    let mut n = self.num_games_created.lock().await;
                    *n += 1;
                }
            }
            Ok(())
        } else {
            Err(())
        }
    }

    async fn get(self, key: Vec<u8>) -> Result<S, ()> {
        let m = self.state_map.lock().await;
        Ok(m.get(&key)
            .cloned()
            .unwrap_or_else(|| (S::new_from_key(key), Instant::now()))
            .0)
    }

    async fn execute_operation_with_messages<E2, F>(
        self,
        key: Vec<u8>,
        operation: F,
    ) -> Result<u64, E2>
    where
        E2: From<()>,
        F: FnOnce(S) -> Result<(S, Vec<S::Message>), E2> + Send + 'static,
    {
        let mut m = self.state_map.lock().await;
        let s = m
            .get(&key)
            .cloned()
            .unwrap_or_else(|| (S::new_from_key(key.clone()), Instant::now()));
        // We're holding the lock, so nobody can actually contend with us. So,
        // we don't need to compare-and-set the relevant version.
        let old_v = s.0.version();
        let (new_state, messages) = operation(s.0)?;
        let new_v = new_state.version();
        if new_v != old_v {
            if !m.contains_key(&key) {
                *self.num_games_created.lock().await += 1;
                info!(self.logger, "Initializing state"; "key" => stringify(&key));
            }
            m.insert(key.clone(), (new_state, Instant::now()));
        }
        drop(m);

        let mut s = self.subscribers.lock().await;
        for m in messages {
            Self::publish(&mut *s, &key, m);
        }
        Ok(new_v)
    }

    async fn subscribe(
        self,
        key: Vec<u8>,
        subscriber_id: usize,
    ) -> Result<mpsc::UnboundedReceiver<S::Message>, ()> {
        info!(self.logger, "Subscribing listener"; "key" => stringify(&key), "subscriber_id" => subscriber_id);
        let mut s = self.subscribers.lock().await;
        let (tx, rx) = mpsc::unbounded_channel();
        let ss = s.entry(key).or_default();
        ss.insert(subscriber_id, tx);
        Ok(rx)
    }

    async fn publish(self, key: Vec<u8>, message: S::Message) -> Result<(), ()> {
        let mut s = self.subscribers.lock().await;
        Self::publish(&mut *s, &key, message);
        Ok(())
    }

    async fn publish_to_single_subscriber(
        self,
        key: Vec<u8>,
        subscriber_id: usize,
        message: S::Message,
    ) -> Result<(), ()> {
        let s = self.subscribers.lock().await;
        if let Some(sender) = s.get(&key).and_then(|ss| ss.get(&subscriber_id)) {
            sender.send(message).map(|_| ()).map_err(|_| ())
        } else {
            Err(())
        }
    }

    async fn unsubscribe(self, key: Vec<u8>, subscriber_id: usize) {
        info!(self.logger, "Unsubscribing listener"; "key" => stringify(&key), "subscriber_id" => subscriber_id);
        let mut m = self.state_map.lock().await;
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
            m.remove(&key);
        }
    }

    async fn get_all_keys(self) -> Result<Vec<Vec<u8>>, ()> {
        let m = self.state_map.lock().await;
        Ok(m.keys().map(|k| k.to_vec()).collect())
    }

    async fn get_states_created(self) -> Result<u64, ()> {
        let n = self.num_games_created.lock().await;
        Ok(*n)
    }

    #[allow(clippy::if_same_then_else)]
    async fn prune(self) {
        // We walk through the key-space and remove any states which are
        // not updated in at least 2 hours.
        // We also remove any subscribers which have disconnected, and
        // subscribers for whom the game is no longer connected.
        let mut m = self.state_map.lock().await;
        let mut s = self.subscribers.lock().await;
        let mut to_prune = vec![];
        for (k, (_, t)) in m.iter() {
            if t.elapsed() > Duration::from_secs(2 * 3600) {
                to_prune.push(k.to_vec());
            } else if s.get(k).map(|ss| ss.is_empty()).unwrap_or(true)
                && t.elapsed() > Duration::from_secs(3600)
            {
                to_prune.push(k.to_vec());
            }
        }
        for k in &to_prune {
            m.remove(k);
            s.remove(k);
        }
        debug!(self.logger, "Ending prune"; "num_states_pruned" => to_prune.len());
    }

    async fn stats(self) -> Result<(usize, usize), ()> {
        let m = self.state_map.lock().await;
        let s = self.subscribers.lock().await;
        Ok((m.len(), s.values().map(|v| v.len()).sum()))
    }
}

fn stringify(str_like: &[u8]) -> &str {
    std::str::from_utf8(str_like).unwrap_or("not utf-8")
}
