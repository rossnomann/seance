use serde::{de::DeserializeOwned, Serialize};
use serde_json::Error as JsonError;
use std::time::{SystemTime, SystemTimeError};

pub(crate) fn now() -> Result<u64, SystemTimeError> {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|x| x.as_secs())
}

pub(super) fn encode_value<V: Serialize>(value: &V) -> Result<Vec<u8>, JsonError> {
    serde_json::to_vec(value)
}

pub(super) fn decode_value<V: DeserializeOwned>(value: &[u8]) -> Result<V, JsonError> {
    serde_json::from_slice(value)
}
