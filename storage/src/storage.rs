use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::mpsc;

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
