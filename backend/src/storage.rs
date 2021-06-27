use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::{mpsc, Mutex};

pub trait State: Serialize + DeserializeOwned + Clone + Send + Default {
    /// Messages that can be sent by operations applied to the state.
    type Message: Serialize + DeserializeOwned + Clone + Send;

    /// The key that this state corresponds to.
    fn key(&self) -> &[u8];
    /// The version of the state. Changes to state require changes in the
    /// version. The default version number must be zero.
    fn version(&self) -> u64;
}

#[async_trait(?Send)]
pub trait Storage<S: State, E> {
    /// Put the state into storage, overwriting any existing value.
    async fn put(&self, state: S) -> Result<(), E>;

    /// Put the state into storage. If the version on the server doesn't match
    /// the expected version, return an error.
    async fn put_cas(&self, expected_version: u64, state: S) -> Result<(), E>;

    /// Get the state corresponding to the key from storage. If it doesn't
    /// exist, a new state will be instantiated with a default version.
    async fn get(&self, key: &[u8]) -> Result<S, E>;

    /// Execute the provided operation based off of the version of the state.
    ///
    /// If the operation succeeds, the returned messages will be published to
    /// the key and the corresponding state will be stored.
    ///
    /// If the operation fails, state will not be changed, and the error will be
    /// returned.
    ///
    /// This operation may also fail if the stored state's `version` field
    /// differs from the one which is fetched at the beginning of the operation
    /// -- it has compare-and-set semantics.
    async fn execute_operation_with_messages<E2>(
        &self,
        key: &[u8],
        operation: impl FnOnce(S) -> Result<(S, Vec<S::Message>), E2> + 'async_trait,
    ) -> Result<(), E2>
    where
        E2: From<E>;

    /// Subscribe to messages about a given key.
    async fn subscribe(&self, key: &[u8]) -> Result<mpsc::UnboundedReceiver<S::Message>, E>;
    /// Subscribe to messages about a given key.
    async fn publish(&self, key: &[u8], message: S::Message) -> Result<(), E>;
}

pub struct HashMapStorage<S: State> {
    state_map: Arc<Mutex<HashMap<Vec<u8>, S>>>,
    subscribers: Arc<Mutex<HashMap<Vec<u8>, Vec<mpsc::UnboundedSender<S::Message>>>>>,
    _data: PhantomData<S>,
}

impl<S: State> HashMapStorage<S> {
    #[allow(unused)]
    pub fn new() -> Self {
        Self {
            state_map: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            _data: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<S: State> Storage<S, ()> for HashMapStorage<S> {
    async fn put(&self, state: S) -> Result<(), ()> {
        let mut m = self.state_map.lock().await;
        m.insert(state.key().to_vec(), state);
        Ok(())
    }

    async fn put_cas(&self, expected_version: u64, state: S) -> Result<(), ()> {
        let mut m = self.state_map.lock().await;
        if m.get(state.key()).map(|s| s.version()).unwrap_or(0) == expected_version {
            if state.version() != expected_version {
                m.insert(state.key().to_vec(), state);
            }
            Ok(())
        } else {
            Err(())
        }
    }

    async fn get(&self, key: &[u8]) -> Result<S, ()> {
        let m = self.state_map.lock().await;
        Ok(m.get(key).cloned().unwrap_or_default())
    }

    async fn execute_operation_with_messages<E2>(
        &self,
        key: &[u8],
        operation: impl FnOnce(S) -> Result<(S, Vec<S::Message>), E2> + 'async_trait,
    ) -> Result<(), E2>
    where
        E2: From<()>,
    {
        let key = key.to_vec();
        let s = self.get(&key).await?;
        let v = s.version();
        let (new_state, messages) = operation(s)?;
        self.put_cas(v, new_state).await?;
        for m in messages {
            self.publish(&key, m).await?;
        }
        Ok(())
    }

    async fn subscribe(&self, key: &[u8]) -> Result<mpsc::UnboundedReceiver<S::Message>, ()> {
        let mut s = self.subscribers.lock().await;
        let (tx, rx) = mpsc::unbounded_channel();
        let ss = s.entry(key.to_vec()).or_default();
        ss.push(tx);
        Ok(rx)
    }

    async fn publish(&self, key: &[u8], message: S::Message) -> Result<(), ()> {
        let mut s = self.subscribers.lock().await;
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
        Ok(())
    }
}
