use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
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

    /// Subscribe to messages about a given key.
    async fn subscribe(self, key: Vec<u8>) -> Result<mpsc::UnboundedReceiver<S::Message>, E>;
    /// Subscribe to messages about a given key.
    async fn publish(self, key: Vec<u8>, message: S::Message) -> Result<(), E>;

    /// Get all of the keys stored in this storage backend.
    async fn get_all_keys(self) -> Result<Vec<Vec<u8>>, E>;
    /// Get the number of states that have been newly created.
    async fn get_states_created(self) -> Result<u64, E>;
}

pub struct HashMapStorage<S: State> {
    state_map: Arc<Mutex<HashMap<Vec<u8>, S>>>,
    subscribers: Arc<Mutex<HashMap<Vec<u8>, Vec<mpsc::UnboundedSender<S::Message>>>>>,
    num_games_created: Arc<Mutex<u64>>,
    _data: PhantomData<S>,
}

impl<S: State> HashMapStorage<S> {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            state_map: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            num_games_created: Arc::new(Mutex::new(0)),
            _data: PhantomData,
        }
    }

    fn publish(
        s: &mut HashMap<Vec<u8>, Vec<mpsc::UnboundedSender<S::Message>>>,
        key: &[u8],
        message: S::Message,
    ) {
        let to_remove = if let Some(subscribers) = s.get_mut(key) {
            let mut send_failed = false;
            for subscriber in subscribers.iter_mut() {
                if subscriber.send(message.clone()).is_err() {
                    send_failed |= true;
                }
            }
            if send_failed {
                subscribers.retain(|subscriber| !subscriber.is_closed());
            }
            subscribers.is_empty()
        } else {
            false
        };
        if to_remove {
            s.remove(key);
        }
    }
}

impl<S: State> Clone for HashMapStorage<S> {
    fn clone(&self) -> Self {
        Self {
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
        m.insert(state.key().to_vec(), state);
        Ok(())
    }

    async fn put_cas(self, expected_version: u64, state: S) -> Result<(), ()> {
        let mut m = self.state_map.lock().await;
        if m.get(state.key()).map(|s| s.version()).unwrap_or(0) == expected_version {
            if state.version() != expected_version {
                m.insert(state.key().to_vec(), state);
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
        Ok(m.get(&key).cloned().unwrap_or_else(|| S::new_from_key(key)))
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
            .unwrap_or_else(|| S::new_from_key(key.clone()));
        // We're holding the lock, so nobody can actually contend with us. So,
        // we don't need to compare-and-set the relevant version.
        let old_v = s.version();
        let (new_state, messages) = operation(s)?;
        let new_v = new_state.version();
        if new_v != old_v {
            m.insert(key.clone(), new_state);
        }
        drop(m);

        let mut s = self.subscribers.lock().await;
        for m in messages {
            Self::publish(&mut *s, &key, m);
        }
        Ok(new_v)
    }

    async fn subscribe(self, key: Vec<u8>) -> Result<mpsc::UnboundedReceiver<S::Message>, ()> {
        let mut s = self.subscribers.lock().await;
        let (tx, rx) = mpsc::unbounded_channel();
        let ss = s.entry(key).or_default();
        ss.push(tx);
        Ok(rx)
    }

    async fn publish(self, key: Vec<u8>, message: S::Message) -> Result<(), ()> {
        let mut s = self.subscribers.lock().await;
        Self::publish(&mut *s, &key, message);
        Ok(())
    }

    async fn get_all_keys(self) -> Result<Vec<Vec<u8>>, ()> {
        let s = self.state_map.lock().await;
        Ok(s.keys().map(|k| k.to_vec()).collect())
    }

    async fn get_states_created(self) -> Result<u64, ()> {
        let s = self.num_games_created.lock().await;
        Ok(*s)
    }
}
