use std::{error::Error, future::Future};

/// Filesystem backend
#[cfg_attr(nightly, doc(cfg(feature = "fs-backend")))]
#[cfg(feature = "fs-backend")]
pub mod fs;

/// Redis backend
#[cfg_attr(nightly, doc(cfg(feature = "redis-backend")))]
#[cfg(feature = "redis-backend")]
pub mod redis;

/// A session backend interface
pub trait SessionBackend {
    /// An error occurred in backend
    type Error: Error + Send + Sync + 'static;

    /// Returns a list of available session IDs
    fn get_sessions(&mut self) -> impl Future<Output = Result<Vec<String>, Self::Error>> + Send;

    /// Returns the time when session was created in seconds
    ///
    /// This method MUST return session age if session exists and None otherwise
    ///
    /// # Arguments
    ///
    /// * session_id - ID of a session
    fn get_session_age(&mut self, session_id: &str) -> impl Future<Output = Result<Option<u64>, Self::Error>> + Send;

    /// Removes a session
    ///
    /// # Arguments
    ///
    /// * session_id - ID of a session
    fn remove_session(&mut self, session_id: &str) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Read a value from store
    ///
    /// * session_id - ID of a session
    /// * key - Key to read value from
    fn read_value(
        &mut self,
        session_id: &str,
        key: &str,
    ) -> impl Future<Output = Result<Option<Vec<u8>>, Self::Error>> + Send;

    /// Write a value to store
    ///
    /// # Arguments
    ///
    /// * session_id - ID of a session
    /// * key - Key to write value to
    /// * value - Value to write
    fn write_value(
        &mut self,
        session_id: &str,
        key: &str,
        value: &[u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Remove a value from store
    ///
    /// * session_id - ID of a session
    /// * key - Key to read value from
    fn remove_value(&mut self, session_id: &str, key: &str) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
