use crate::{backend::SessionBackend, utils::now};
use async_trait::async_trait;
use darkredis::{Command, ConnectionPool, Error as DarkredisError};
use snafu::{ResultExt, Snafu};
use std::{num::ParseIntError, string::FromUtf8Error, time::SystemTimeError};

/// Redis powered session backend
#[derive(Clone)]
pub struct RedisBackend {
    namespace: String,
    sessions_key: String,
    pool: ConnectionPool,
}

impl RedisBackend {
    /// Creates a new backend
    ///
    /// # Arguments
    ///
    /// * namespace - A prefix string for keys
    /// * pool - A redis connection pool
    pub fn new<N>(namespace: N, pool: ConnectionPool) -> Self
    where
        N: Into<String>,
    {
        let namespace = namespace.into();
        let sessions_key = format!("{}:__seance_sessions", namespace);
        Self {
            namespace,
            sessions_key,
            pool,
        }
    }

    fn get_session_key(&self, session_id: &str) -> String {
        format!("{}:{}", self.namespace, session_id)
    }
}

#[async_trait]
impl SessionBackend for RedisBackend {
    type Error = RedisError;

    async fn get_sessions(&mut self) -> Result<Vec<String>, Self::Error> {
        let mut connection = self.pool.get().await;
        let value = connection
            .run_command(Command::new("HKEYS").arg(&self.sessions_key))
            .await
            .context(GetSessions)?;
        let mut result = Vec::new();
        for session_id in value.unwrap_array() {
            result.push(String::from_utf8(session_id.unwrap_string()).context(ParseSessionId)?);
        }
        Ok(result)
    }

    async fn get_session_age(&mut self, session_id: &str) -> Result<Option<u64>, Self::Error> {
        let mut connection = self.pool.get().await;
        let value = connection
            .run_command(
                Command::new("HGET")
                    .arg(&self.sessions_key)
                    .arg(&session_id),
            )
            .await
            .context(GetSessionAge)?;
        Ok(match value.optional_string() {
            Some(value) => Some(
                String::from_utf8(value)
                    .context(SessionAgeFromUtf8)?
                    .parse::<u64>()
                    .context(ParseSessionAge)?,
            ),
            None => None,
        })
    }

    async fn remove_session(&mut self, session_id: &str) -> Result<(), Self::Error> {
        let session_key = self.get_session_key(session_id);
        let mut connection = self.pool.get().await;
        connection.del(session_key).await.context(RemoveSession)?;
        Ok(())
    }

    async fn read_value(
        &mut self,
        session_id: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let session_key = self.get_session_key(session_id);
        let mut connection = self.pool.get().await;
        let value = connection
            .run_command(Command::new("HGET").arg(&session_key).arg(&key))
            .await
            .context(ReadValue)?;
        Ok(value.optional_string())
    }

    async fn write_value(
        &mut self,
        session_id: &str,
        key: &str,
        value: &[u8],
    ) -> Result<(), Self::Error> {
        let session_key = self.get_session_key(session_id);
        let mut connection = self.pool.get().await;
        let len = connection
            .run_command(Command::new("HLEN").arg(&session_key))
            .await
            .context(WriteValue)?;
        if len.unwrap_integer() == 0 {
            let timestamp = format!("{}", now().context(SetSessionTimestamp)?);
            connection
                .run_command(
                    Command::new("HSET")
                        .arg(&self.sessions_key)
                        .arg(&session_id)
                        .arg(&timestamp),
                )
                .await
                .context(WriteValue)?;
        }
        connection
            .run_command(Command::new("HSET").arg(&session_key).arg(&key).arg(&value))
            .await
            .context(WriteValue)?;
        Ok(())
    }

    async fn remove_value(&mut self, session_id: &str, key: &str) -> Result<(), Self::Error> {
        let session_key = self.get_session_key(session_id);
        let mut connection = self.pool.get().await;
        connection
            .run_command(Command::new("HDEL").arg(&session_key).arg(&key))
            .await
            .context(RemoveValue)?;
        Ok(())
    }
}

/// An error occurred in redis backend
#[derive(Debug, Snafu)]
pub enum RedisError {
    /// Failed to get sessions list
    #[snafu(display("failed to get sessions list: {}", source))]
    GetSessions {
        /// Source error
        source: DarkredisError,
    },

    /// Failed to get session age
    #[snafu(display("failed to get session age: {}", source))]
    GetSessionAge {
        /// Source error
        source: DarkredisError,
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
        source: DarkredisError,
    },

    /// Failed to remove session
    #[snafu(display("failed to remove session: {}", source))]
    RemoveSession {
        /// Source error
        source: DarkredisError,
    },

    /// Failed to remove value
    #[snafu(display("failed to remove value: {}", source))]
    RemoveValue {
        /// Source error
        source: DarkredisError,
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
        source: DarkredisError,
    },
}
