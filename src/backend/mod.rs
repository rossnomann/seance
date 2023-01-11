use std::error::Error;

use async_trait::async_trait;

/// Filesystem backend
///
/// Available with enabled `fs-backend` feature
#[cfg(feature = "fs-backend")]
pub mod fs;

/// Redis backend
///
/// Available with enabled `redis-backend` feature
#[cfg(feature = "redis-backend")]
pub mod redis;

/// A session backend interface
#[async_trait]
pub trait SessionBackend {
    /// An error occurred in backend
    type Error: Error + Send + Sync + 'static;

    /// Returns a list of available session IDs
    async fn get_sessions(&mut self) -> Result<Vec<String>, Self::Error>;

    /// Returns the time when session was created in seconds
    ///
    /// This method MUST return session age if session exists and None otherwise
    ///
    /// # Arguments
    ///
    /// * session_id - ID of a session
    async fn get_session_age(&mut self, session_id: &str) -> Result<Option<u64>, Self::Error>;

    /// Removes a session
    ///
    /// # Arguments
    ///
    /// * session_id - ID of a session
    async fn remove_session(&mut self, session_id: &str) -> Result<(), Self::Error>;

    /// Read a value from store
    ///
    /// * session_id - ID of a session
    /// * key - Key to read value from
    async fn read_value(&mut self, session_id: &str, key: &str) -> Result<Option<Vec<u8>>, Self::Error>;

    /// Write a value to store
    ///
    /// # Arguments
    ///
    /// * session_id - ID of a session
    /// * key - Key to write value to
    /// * value - Value to write
    async fn write_value(&mut self, session_id: &str, key: &str, value: &[u8]) -> Result<(), Self::Error>;

    /// Remove a value from store
    ///
    /// * session_id - ID of a session
    /// * key - Key to read value from
    async fn remove_value(&mut self, session_id: &str, key: &str) -> Result<(), Self::Error>;
}
