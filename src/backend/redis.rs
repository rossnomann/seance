use crate::{backend::SessionBackend, utils::now};
use async_trait::async_trait;
use redis::{AsyncCommands, RedisError};
use snafu::{ResultExt, Snafu};
use std::{num::ParseIntError, string::FromUtf8Error, time::SystemTimeError};

/// Redis powered session backend
#[derive(Clone)]
pub struct RedisBackend<C> {
    namespace: String,
    sessions_key: String,
    connection: C,
}

impl<C> RedisBackend<C> {
    /// Creates a new backend
    ///
    /// # Arguments
    ///
    /// * namespace - A prefix string for keys
    /// * connection - A redis connection manager
    pub fn new<N>(namespace: N, connection: C) -> Self
    where
        N: Into<String>,
    {
        let namespace = namespace.into();
        let sessions_key = format!("{}:__seance_sessions", namespace);
        Self {
            namespace,
            sessions_key,
            connection,
        }
    }

    fn get_session_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.namespace, session_id)
    }
}

#[async_trait]
impl<C> SessionBackend for RedisBackend<C>
where
    C: AsyncCommands,
{
    type Error = RedisBackendError;

    async fn get_sessions(&mut self) -> Result<Vec<String>, Self::Error> {
        Ok(self
            .connection
            .hkeys(&self.sessions_key)
            .await
            .context(GetSessions)?)
    }

    async fn get_session_age(&mut self, session_id: &str) -> Result<Option<u64>, Self::Error> {
        Ok(self
            .connection
            .hget(&self.sessions_key, session_id)
            .await
            .context(GetSessionAge)?)
    }

    async fn remove_session(&mut self, session_id: &str) -> Result<(), Self::Error> {
        let session_key = self.get_session_key(session_id);
        self.connection
            .del(session_key)
            .await
            .context(RemoveSession)?;
        Ok(())
    }

    async fn read_value(
        &mut self,
        session_id: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let session_key = self.get_session_key(session_id);
        Ok(self
            .connection
            .hget(session_key, key)
            .await
            .context(ReadValue)?)
    }

    async fn write_value(
        &mut self,
        session_id: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), Self::Error> {
        let session_key = self.get_session_key(session_id);
        let len: i64 = self
            .connection
            .hlen(&session_key)
            .await
            .context(WriteValue)?;
        if len == 0 {
            let timestamp = format!("{}", now().context(SetSessionTimestamp)?);
            self.connection
                .hset(&self.sessions_key, session_id, timestamp)
                .await
                .context(WriteValue)?;
        }
        self.connection
            .hset(session_key, key, value)
            .await
            .context(WriteValue)?;
        Ok(())
    }

    async fn remove_value(&mut self, session_id: &str, key: &str) -> Result<(), Self::Error> {
        let session_key = self.get_session_key(session_id);
        self.connection
            .hdel(session_key, key)
            .await
            .context(RemoveValue)?;
        Ok(())
    }
}

/// An error occurred in redis backend
#[derive(Debug, Snafu)]
pub enum RedisBackendError {
    /// Failed to get sessions list
    #[snafu(display("failed to get sessions list: {}", source))]
    GetSessions {
        /// Source error
        source: RedisError,
    },

    /// Failed to get session age
    #[snafu(display("failed to get session age: {}", source))]
    GetSessionAge {
        /// Source error
        source: RedisError,
    },

    /// Failed to parse session age
    #[snafu(display("session age contains non-integer value: {}", source))]
    ParseSessionAge {
        /// Source error
        source: ParseIntError,
    },

    /// Failed to parse session ID
    #[snafu(display("session id contains non-utf8 string: {}", source))]
    ParseSessionId {
        /// Source error
        source: FromUtf8Error,
    },

    /// Failed to read value
    #[snafu(display("failed to read value: {}", source))]
    ReadValue {
        /// Source error
        source: RedisError,
    },

    /// Failed to remove session
    #[snafu(display("failed to remove session: {}", source))]
    RemoveSession {
        /// Source error
        source: RedisError,
    },

    /// Failed to remove value
    #[snafu(display("failed to remove value: {}", source))]
    RemoveValue {
        /// Source error
        source: RedisError,
    },

    /// Failed to read session age
    #[snafu(display("session age contains non-utf8 string: {}", source))]
    SessionAgeFromUtf8 {
        /// Source error
        source: FromUtf8Error,
    },

    /// Failed to set session timestamp
    ///
    /// An error occurred when getting system time
    #[snafu(display("failed to set session timestamp: {}", source))]
    SetSessionTimestamp {
        /// Source error
        source: SystemTimeError,
    },

    /// Failed to write value
    #[snafu(display("failed to write value: {}", source))]
    WriteValue {
        /// Source error
        source: RedisError,
    },
}
