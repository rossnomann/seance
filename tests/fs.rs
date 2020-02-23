use seance::{backend::fs::FilesystemBackend, SessionCollector, SessionManager};
use std::time::Duration;
use tempfile::tempdir;
use tokio::time::delay_for;

#[tokio::test]
async fn fs() {
    let tmpdir = tempdir().expect("Failed to create temp directory");
    let backend = FilesystemBackend::new(tmpdir.into_path());
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
