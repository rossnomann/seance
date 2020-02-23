use darkredis::ConnectionPool;
use seance::{backend::redis::RedisBackend, SessionCollector, SessionManager};
use std::{
    env::{var, VarError},
    time::Duration,
};
use tokio::time::delay_for;

const CONNECTION_COUNT: usize = 1;
const DEFAULT_ADDRESS: &str = "127.0.0.1:6379";

#[tokio::test]
async fn redis() {
    let address = match var("SEANCE_REDIS_ADDRESS") {
        Ok(address) => address,
        Err(VarError::NotPresent) => String::from(DEFAULT_ADDRESS),
        Err(err) => panic!("{}", err),
    };
    println!("REDIS ADDRESS: {:?}", address);
    let pool = ConnectionPool::create(address, None, CONNECTION_COUNT)
        .await
        .unwrap();
    let backend = RedisBackend::new("test-seance", pool);
    let manager = SessionManager::new(backend.clone());
    let mut session = manager.get_session("session-id");
    session.set("key", &"value").await.unwrap();
    assert_eq!(
        "value",
        session.get::<_, String>("key").await.unwrap().unwrap()
    );
    session.remove("key").await.unwrap();
    assert!(session.get::<_, String>("key").await.unwrap().is_none());
    session.set("key", &"value").await.unwrap();
    session.expire("key", 1).await.unwrap();
    delay_for(Duration::from_secs(2)).await;
    assert!(session.get::<_, String>("key").await.unwrap().is_none());

    let gc_period = Duration::from_secs(1);
    let session_lifetime = Duration::from_secs(1);
    let mut collector = SessionCollector::new(backend, gc_period, session_lifetime);
    let handle = collector.get_handle();
    session.set("key", &"value").await.unwrap();
    tokio::spawn(async move {
        collector.run().await;
    });
    delay_for(Duration::from_secs(2)).await;
    handle.shutdown().await;
    assert!(session.get::<_, String>("key").await.unwrap().is_none());
}
