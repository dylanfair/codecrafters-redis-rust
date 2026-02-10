use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type RedisCache = Arc<Mutex<HashMap<String, RedisValue>>>;

pub enum ExpirationFidelity {
    Ex(u64),
    Px(u64),
    Exat(u64),
    Pxat(u64),
}

#[derive(Debug)]
pub enum DataType {
    String(String),
    List(Vec<String>),
}

#[derive(Debug)]
pub struct RedisValue {
    pub value: DataType,
    pub expiration: Option<DateTime<Utc>>,
}

impl RedisValue {
    pub fn new(value: DataType, expiration: Option<ExpirationFidelity>) -> RedisValue {
        let expiration = if let Some(expiration_type) = expiration {
            match expiration_type {
                ExpirationFidelity::Ex(duration) => {
                    Some(Utc::now() + Duration::seconds(duration as i64))
                }
                ExpirationFidelity::Px(duration) => {
                    Some(Utc::now() + Duration::milliseconds(duration as i64))
                }
                ExpirationFidelity::Exat(timestamp) => {
                    DateTime::from_timestamp_secs(timestamp as i64)
                }
                ExpirationFidelity::Pxat(timestamp) => {
                    DateTime::from_timestamp_millis(timestamp as i64)
                }
            }
        } else {
            None
        };
        RedisValue { value, expiration }
    }

    pub fn append_to_list(&mut self, value: String) -> Result<usize> {
        match &mut self.value {
            DataType::List(existing_list) => {
                existing_list.push(value);
                Ok(existing_list.len())
            }
            DataType::String(_) => Err(anyhow!("Trying to append to a string datatype")),
        }
    }
}
