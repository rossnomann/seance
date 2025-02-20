use crate::utils::now;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Error as JsonError, Value as JsonValue};
use std::time::SystemTimeError;

/// Wrapper for an owned session value
#[derive(Serialize, Deserialize)]
pub struct Value {
    expires_at: Option<u64>,
    value: JsonValue,
}

impl Value {
    /// Returns a parsed value
    pub fn into_parsed<T: DeserializeOwned>(mut self) -> Result<T, JsonError> {
        serde_json::from_value(self.value.take())
    }

    /// Sets value lifetime in seconds from now
    pub fn set_lifetime(&mut self, lifetime: u64) -> Result<(), SystemTimeError> {
        self.expires_at = Some(now()? + lifetime);
        Ok(())
    }

    /// Returns UNIX timestamp when the value expires
    pub fn get_expires_at(&self) -> Option<u64> {
        self.expires_at
    }

    /// Whether value expired
    pub fn is_expired(&self) -> Result<bool, SystemTimeError> {
        let timestamp = now()?;
        Ok(self
            .expires_at
            .map(|expires_at| expires_at < timestamp)
            .unwrap_or(false))
    }
}

/// Wrapper for a session value reference
#[derive(Serialize)]
pub struct ValueRef<'a, T>
where
    T: Serialize,
{
    expires_at: Option<u64>,
    value: &'a T,
}

impl<'a, T> ValueRef<'a, T>
where
    T: Serialize + 'a,
{
    /// Creates a new value
    pub fn new(value: &'a T) -> Self {
        Self {
            value,
            expires_at: None,
        }
    }

    /// Sets UNIX timestamp when a value should expire
    pub fn set_expires_at(&mut self, expires_at: u64) {
        self.expires_at = Some(expires_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_value() {
        let value: Value = serde_json::from_value(serde_json::json!({
            "expires_at": 0,
            "value": "test",
        }))
        .unwrap();
        assert_eq!(value.get_expires_at().unwrap(), 0);
        assert!(value.is_expired().unwrap());
        assert_eq!(value.into_parsed::<String>().unwrap(), "test");

        let value: Value = serde_json::from_value(serde_json::json!({
            "value": "test",
        }))
        .unwrap();
        assert!(value.get_expires_at().is_none());
        assert!(!value.is_expired().unwrap());
        assert_eq!(value.into_parsed::<String>().unwrap(), "test");
    }

    #[test]
    fn serialize_value() {
        let mut value: Value = serde_json::from_value(serde_json::json!({
            "expires_at": 0,
            "value": "test",
        }))
        .unwrap();
        value.set_lifetime(100).unwrap();
        assert!(value.get_expires_at().unwrap() > now().unwrap());
        value.expires_at = Some(100);
        assert_eq!(value.get_expires_at().unwrap(), 100);
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            r#"{"expires_at":100,"value":"test"}"#
        );

        let value: Value = serde_json::from_value(serde_json::json!({"value": "test"})).unwrap();
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            r#"{"expires_at":null,"value":"test"}"#
        );
    }

    #[test]
    fn serialize_value_ref() {
        let mut value = ValueRef::new(&"testref");
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            r#"{"expires_at":null,"value":"testref"}"#
        );
        value.set_expires_at(100);
        assert_eq!(
            serde_json::to_string(&value).unwrap(),
            r#"{"expires_at":100,"value":"testref"}"#
        );
    }
}
