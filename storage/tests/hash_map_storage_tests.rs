use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use slog::{o, Drain, Logger};

use storage::{HashMapStorage, State, Storage};
use tokio::sync::oneshot;
use tokio::task;

struct NoOpDrain;

impl Drain for NoOpDrain {
    type Ok = ();
    type Err = ();
    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        println!("{:?}, {:?}", record.msg(), values);
        Ok(())
    }
}

fn make_logger() -> Logger {
    let drain = Mutex::new(NoOpDrain).fuse();
    Logger::root(drain, o!())
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct VersionedState {
    key: Vec<u8>,
    version: u64,
}

impl State for VersionedState {
    type Message = ();

    fn key(&self) -> &[u8] {
        &self.key
    }
    fn version(&self) -> u64 {
        self.version
    }
    fn new_from_key(key: Vec<u8>) -> Self {
        Self { key, version: 0 }
    }
}

macro_rules! vs {
    ($key: expr, $version: expr) => {
        VersionedState {
            key: $key.as_bytes().to_vec(),
            version: $version,
        }
    };
}

#[tokio::test]
async fn test_basic_kv() {
    let s: HashMapStorage<VersionedState> = HashMapStorage::new(make_logger());

    // Get a non-existent value
    assert_eq!(
        s.clone().get(b"test".to_vec()).await.unwrap(),
        vs!("test", 0)
    );

    // Put a real value there.
    s.clone().put(vs!("test", 1)).await.unwrap();

    // Try to retrieve it
    assert_eq!(
        s.clone().get(b"test".to_vec()).await.unwrap(),
        vs!("test", 1)
    );

    // Try to race with compare-and-set
    s.clone().put_cas(0, vs!("test", 2)).await.unwrap_err();

    // Try to successfully compare-and-set
    s.clone().put_cas(1, vs!("test", 2)).await.unwrap();

    // Validate that we can fetch all the keys
    assert_eq!(
        s.clone().get_all_keys().await.unwrap(),
        vec![b"test".to_vec()]
    );

    // Validate that we only incremented the number of created-states once.
    assert_eq!(s.clone().get_states_created().await.unwrap(), 1);

    // Validate that the stats are correct.
    assert_eq!(s.clone().stats().await.unwrap(), (1, 0));
}

#[tokio::test]
async fn test_basic_pubsub() {
    let s: HashMapStorage<VersionedState> = HashMapStorage::new(make_logger());
    let mut subscription = s.clone().subscribe(b"test".to_vec(), 0).await.unwrap();

    let handle = task::spawn(async move {
        let mut count = 0usize;
        while let Some(_) = subscription.recv().await {
            count += 1;
        }
        count
    });

    // Publish a general message
    s.clone().publish(b"test".to_vec(), ()).await.unwrap();
    // Publish a message to a specific subscriber by ID
    s.clone()
        .publish_to_single_subscriber(b"test".to_vec(), 0, ())
        .await
        .unwrap();
    // Publish to a different subscriber in the same room by ID.
    s.clone()
        .publish_to_single_subscriber(b"test".to_vec(), 1, ())
        .await
        .unwrap_err();

    assert_eq!(s.clone().stats().await.unwrap(), (0, 1));

    // Unsubscribe the subscriber, which should allow the handle to join successfully.
    s.clone().unsubscribe(b"test".to_vec(), 0).await;

    let num_messages = handle.await.unwrap();
    assert_eq!(num_messages, 2);

    // Try again, but this time the subscriber will go away.

    let mut subscription = s.clone().subscribe(b"test".to_vec(), 0).await.unwrap();
    let (tx, mut rx) = oneshot::channel();
    let (tx1, rx1) = oneshot::channel();

    let handle = task::spawn(async move {
        let mut tx1 = Some(tx1);
        loop {
            tokio::select! {
                _ = &mut rx => {
                    break;
                }
                v = subscription.recv() => {
                    if v.is_none() {
                        break;
                    } else if let Some(tx1) = tx1.take() {
                        tx1.send(()).unwrap();
                    }
                }
            }
        }
    });

    s.clone().publish(b"test".to_vec(), ()).await.unwrap();
    // Publish a message. After it's received, we'll get a response on rx1.
    rx1.await.unwrap();

    // Send the message directly to the consumer so that it drops.
    tx.send(()).unwrap();
    handle.await.unwrap();

    // Try to send a message to it via publish again.
    s.clone().publish(b"test".to_vec(), ()).await.unwrap();

    // Check that there are no subscribers
    assert_eq!(s.clone().stats().await.unwrap(), (0, 0));
}

#[tokio::test]
async fn test_execute_operation() {
    let s: HashMapStorage<VersionedState> = HashMapStorage::new(make_logger());
    let mut num_expected_messages = 0;

    // Execute an operation with no subscribers and which has no impact.
    s.clone()
        .execute_operation_with_messages::<(), _>(b"test".to_vec(), |existing_state| {
            assert_eq!(existing_state, vs!("test", 0));
            Ok((existing_state, vec![()]))
        })
        .await
        .unwrap();

    // Add a subscriber
    let mut subscription = s.clone().subscribe(b"test".to_vec(), 0).await.unwrap();

    let handle = task::spawn(async move {
        let mut count = 0usize;
        while let Some(_) = subscription.recv().await {
            count += 1;
        }
        count
    });

    // Execute the no-op operation again, but this time we have a subscriber.
    num_expected_messages += 1;
    s.clone()
        .execute_operation_with_messages::<(), _>(b"test".to_vec(), |existing_state| {
            assert_eq!(existing_state, vs!("test", 0));
            Ok((existing_state, vec![()]))
        })
        .await
        .unwrap();

    // Try it with multiple messages.
    num_expected_messages += 2;
    s.clone()
        .execute_operation_with_messages::<(), _>(b"test".to_vec(), |existing_state| {
            assert_eq!(existing_state, vs!("test", 0));
            Ok((existing_state, vec![(), ()]))
        })
        .await
        .unwrap();

    // Try it if we just change the state, but don't have any messages
    s.clone()
        .execute_operation_with_messages::<(), _>(b"test".to_vec(), |existing_state| {
            assert_eq!(existing_state, vs!("test", 0));
            Ok((vs!("test", 1), vec![]))
        })
        .await
        .unwrap();

    // Try it if we change the state _and_ leave a message
    num_expected_messages += 1;
    s.clone()
        .execute_operation_with_messages::<(), _>(b"test".to_vec(), |existing_state| {
            assert_eq!(existing_state, vs!("test", 1));
            Ok((vs!("test", 2), vec![()]))
        })
        .await
        .unwrap();

    // Validate that it has the right state at the end.
    assert_eq!(
        s.clone().get(b"test".to_vec()).await.unwrap(),
        vs!("test", 2)
    );

    // Unsubscribe the subscriber, which should allow the handle to join successfully.
    s.clone().unsubscribe(b"test".to_vec(), 0).await;

    let num_messages = handle.await.unwrap();
    assert_eq!(num_messages, num_expected_messages);
}
