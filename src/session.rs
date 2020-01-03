use crate::{
    backend::SessionBackend,
    utils::{decode_value, encode_value},
    value::{Value, ValueRef},
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Error as JsonError;
use std::{error::Error, fmt, sync::Arc, time::SystemTimeError};
use tokio::sync::Mutex;

/// Actual session
#[derive(Clone)]
pub struct Session<B> {
    id: String,
    backend: Arc<Mutex<B>>,
}

impl<B> Session<B>
where
    B: SessionBackend,
{
    pub(crate) fn new<I>(id: I, backend: Arc<Mutex<B>>) -> Self
    where
        I: Into<String>,
    {
        Self {
            id: id.into(),
            backend,
        }
    }

    async fn read_value(&mut self, key: &str) -> Result<Option<Value>, SessionError> {
        let mut backend = self.backend.lock().await;
        match backend
            .read_value(&self.id, key.as_ref())
            .await
            .map_err(SessionError::backend)?
        {
            Some(value) => {
                let value = decode_value(&value).map_err(SessionError::DecodeValue)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn write_value<V: Serialize>(&mut self, key: &str, value: V) -> Result<(), SessionError> {
        let mut backend = self.backend.lock().await;
        let data = encode_value(&value).map_err(SessionError::EncodeValue)?;
        backend
            .write_value(&self.id, key.as_ref(), &data)
            .await
            .map_err(SessionError::backend)?;
        Ok(())
    }

    /// Sets a value for key
    pub async fn set<K, V>(&mut self, key: K, value: &V) -> Result<(), SessionError>
    where
        K: AsRef<str>,
        V: Serialize,
    {
        let key = key.as_ref();
        let mut value = ValueRef::new(&value);
        if let Some(old_value) = self.read_value(key).await? {
            if !old_value.is_expired().map_err(SessionError::CheckExpired)? {
                if let Some(expires_at) = old_value.get_expires_at() {
                    value.set_expires_at(expires_at);
                }
            }
        };
        self.write_value(key, value).await?;
        Ok(())
    }

    /// Gets a value for key
    pub async fn get<K, O>(&mut self, key: K) -> Result<Option<O>, SessionError>
    where
        K: AsRef<str>,
        O: DeserializeOwned,
    {
        Ok(
            if let Some(value) = self
                .read_value(key.as_ref())
                .await
                .map_err(SessionError::backend)?
            {
                if value.is_expired().map_err(SessionError::CheckExpired)? {
                    None
                } else {
                    Some(value.into_parsed().map_err(SessionError::ParseValue)?)
                }
            } else {
                None
            },
        )
    }

    /// Expires a key
    pub async fn expire<K>(&mut self, key: K, seconds: u64) -> Result<(), SessionError>
    where
        K: AsRef<str>,
    {
        let key = key.as_ref();
        if let Some(mut value) = self.read_value(key).await.map_err(SessionError::backend)? {
            value
                .set_lifetime(seconds)
                .map_err(SessionError::ExpireValue)?;
            self.write_value(key, value)
                .await
                .map_err(SessionError::backend)?;
        }
        Ok(())
    }

    /// Removes a key
    pub async fn remove<K>(&mut self, key: K) -> Result<(), SessionError>
    where
        K: AsRef<str>,
    {
        let mut backend = self.backend.lock().await;
        backend
            .remove_value(&self.id, key.as_ref())
            .await
            .map_err(SessionError::backend)
    }
}

/// An error occurred in session
#[derive(Debug)]
pub enum SessionError {
    /// Backend error
    Backend(Box<dyn Error>),
    /// Failed to check whether value expired
    CheckExpired(SystemTimeError),
    /// Failed to decode value
    DecodeValue(JsonError),
    /// Failed to encode value
    EncodeValue(JsonError),
    /// Failed to expire value
    ExpireValue(SystemTimeError),
    /// Failed to parse value
    ParseValue(JsonError),
}

impl SessionError {
    fn backend<E: Error + 'static>(err: E) -> Self {
        Self::Backend(Box::new(err))
    }
}

impl Error for SessionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SessionError::Backend(err) => Some(err.as_ref()),
            SessionError::CheckExpired(err) => Some(err),
            SessionError::DecodeValue(err) => Some(err),
            SessionError::EncodeValue(err) => Some(err),
            SessionError::ExpireValue(err) => Some(err),
            SessionError::ParseValue(err) => Some(err),
        }
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SessionError::Backend(err) => write!(out, "backend error: {}", err),
            SessionError::CheckExpired(err) => {
                write!(out, "failed to check whether value expired: {}", err)
            }
            SessionError::DecodeValue(err) => write!(out, "failed to decode value: {}", err),
            SessionError::EncodeValue(err) => write!(out, "failed to encode value: {}", err),
            SessionError::ExpireValue(err) => write!(out, "failed to expire value: {}", err),
            SessionError::ParseValue(err) => write!(out, "failed to parse value: {}", err),
        }
    }
}
