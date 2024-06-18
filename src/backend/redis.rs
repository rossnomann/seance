use std::{error::Error, fmt, num::ParseIntError, string::FromUtf8Error, time::SystemTimeError};

use redis::{AsyncCommands, RedisError};

use crate::{backend::SessionBackend, utils::now};

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
        let sessions_key = format!("{namespace}:__seance_sessions");
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

impl<C> SessionBackend for RedisBackend<C>
where
    C: AsyncCommands,
{
    type Error = RedisBackendError;

    async fn get_sessions(&mut self) -> Result<Vec<String>, Self::Error> {
        self.connection
            .hkeys(&self.sessions_key)
            .await
            .map_err(RedisBackendError::GetSessions)
    }

    async fn get_session_age(&mut self, session_id: &str) -> Result<Option<u64>, Self::Error> {
        self.connection
            .hget(&self.sessions_key, session_id)
            .await
            .map_err(RedisBackendError::GetSessionAge)
    }

    async fn remove_session(&mut self, session_id: &str) -> Result<(), Self::Error> {
        let session_key = self.get_session_key(session_id);
        self.connection
            .del(session_key)
            .await
            .map_err(RedisBackendError::RemoveSession)
    }

    async fn read_value(&mut self, session_id: &str, key: &str) -> Result<Option<Vec<u8>>, Self::Error> {
        let session_key = self.get_session_key(session_id);
        // Use additional variable because trait bound for FromRedisValue is not satisfied for some reason
        let result: Option<Vec<u8>> = self
            .connection
            .hget(session_key, key)
            .await
            .map_err(RedisBackendError::ReadValue)?;
        Ok(result)
    }

    async fn write_value(&mut self, session_id: &str, key: &str, value: &[u8]) -> Result<(), Self::Error> {
        let session_key = self.get_session_key(session_id);
        let len: i64 = self
            .connection
            .hlen(&session_key)
            .await
            .map_err(RedisBackendError::WriteValue)?;
        if len == 0 {
            let timestamp = format!("{}", now().map_err(RedisBackendError::SetSessionTimestamp)?);
            self.connection
                .hset(&self.sessions_key, session_id, timestamp)
                .await
                .map_err(RedisBackendError::WriteValue)?;
        }
        self.connection
            .hset(session_key, key, value)
            .await
            .map_err(RedisBackendError::WriteValue)
    }

    async fn remove_value(&mut self, session_id: &str, key: &str) -> Result<(), Self::Error> {
        let session_key = self.get_session_key(session_id);
        self.connection
            .hdel(session_key, key)
            .await
            .map_err(RedisBackendError::RemoveValue)
    }
}

/// An error occurred in redis backend
#[derive(Debug)]
pub enum RedisBackendError {
    /// Failed to get sessions list
    GetSessions(RedisError),
    /// Failed to get session age
    GetSessionAge(RedisError),
    /// Failed to parse session age
    ParseSessionAge(ParseIntError),
    /// Failed to parse session ID
    ParseSessionId(FromUtf8Error),
    /// Failed to read value
    ReadValue(RedisError),
    /// Failed to remove session
    RemoveSession(RedisError),
    /// Failed to remove value
    RemoveValue(RedisError),
    /// Failed to read session age
    SessionAgeFromUtf8(FromUtf8Error),
    /// Failed to set session timestamp
    ///
    /// An error occurred when getting system time
    SetSessionTimestamp(SystemTimeError),
    /// Failed to write value
    WriteValue(RedisError),
}

impl fmt::Display for RedisBackendError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::RedisBackendError::*;
        match self {
            GetSessions(err) => write!(out, "failed to get sessions list: {err}"),
            GetSessionAge(err) => write!(out, "failed to get session age: {err}"),
            ParseSessionAge(err) => write!(out, "session age contains non-integer value: {err}"),
            ParseSessionId(err) => write!(out, "session id contains non-utf8 string: {err}"),
            ReadValue(err) => write!(out, "failed to read value: {err}"),
            RemoveSession(err) => write!(out, "failed to remove session: {err}"),
            RemoveValue(err) => write!(out, "failed to remove value: {err}"),
            SessionAgeFromUtf8(err) => write!(out, "session age contains non-utf8 string: {err}"),
            SetSessionTimestamp(err) => write!(out, "failed to set session timestamp: {err}"),
            WriteValue(err) => write!(out, "failed to write value: {err}"),
        }
    }
}

impl Error for RedisBackendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::RedisBackendError::*;
        Some(match self {
            GetSessions(err) => err,
            GetSessionAge(err) => err,
            ParseSessionAge(err) => err,
            ParseSessionId(err) => err,
            ReadValue(err) => err,
            RemoveSession(err) => err,
            RemoveValue(err) => err,
            SessionAgeFromUtf8(err) => err,
            SetSessionTimestamp(err) => err,
            WriteValue(err) => err,
        })
    }
}
