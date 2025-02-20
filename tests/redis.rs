use std::{
    env::{VarError, var},
    time::Duration,
};

use redis::Client;
use tokio::time::sleep;

use seance::{SessionCollector, SessionManager, backend::redis::RedisBackend};

const DEFAULT_ADDRESS: &str = "redis://127.0.0.1:6379";

#[tokio::test]
async fn redis() {
    let address = match var("SEANCE_REDIS_ADDRESS") {
        Ok(address) => address,
        Err(VarError::NotPresent) => String::from(DEFAULT_ADDRESS),
        Err(err) => panic!("{}", err),
    };
    println!("REDIS ADDRESS: {:?}", address);
    let client = Client::open(address).unwrap();
    let manager = client.get_multiplexed_tokio_connection().await.unwrap();
    let backend = RedisBackend::new("test-seance", manager);
    let manager = SessionManager::new(backend.clone());
    let mut session = manager.get_session("session-id");
    session.set("key", &"value").await.unwrap();
    assert_eq!("value", session.get::<_, String>("key").await.unwrap().unwrap());
    session.remove("key").await.unwrap();
    assert!(session.get::<_, String>("key").await.unwrap().is_none());
    session.set("key", &"value").await.unwrap();
    session.expire("key", 1).await.unwrap();
    sleep(Duration::from_secs(2)).await;
    assert!(session.get::<_, String>("key").await.unwrap().is_none());

    let gc_period = Duration::from_secs(1);
    let session_lifetime = Duration::from_secs(1);
    let mut collector = SessionCollector::new(backend, gc_period, session_lifetime);
    let handle = collector.get_handle();
    session.set("key", &"value").await.unwrap();
    tokio::spawn(async move {
        collector.run().await;
    });
    sleep(Duration::from_secs(2)).await;
    handle.shutdown().await;
    assert!(session.get::<_, String>("key").await.unwrap().is_none());
}
