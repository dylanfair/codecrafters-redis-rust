use anyhow::{Result, anyhow};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{Arc, Mutex, MutexGuard};

pub type RedisCache = Arc<Mutex<HashMap<String, RedisValue>>>;

pub enum ExpirationFidelity {
    Ex(u64),
    Px(u64),
    Exat(u64),
    Pxat(u64),
}

#[derive(Debug, Clone)]
pub enum DataType {
    String(String),
    List(Vec<String>),
    Set(Vec<String>),
    Zset(Vec<String>),
    Hash(String),
    Stream(String),
    Vectorset(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct RedisValue {
    pub value: DataType,
    pub expiration: Option<DateTime<Utc>>,
}

impl RedisValue {
    pub fn datatype_str(&self) -> String {
        match self.value {
            DataType::String(_) => "string".to_string(),
            DataType::List(_) => "list".to_string(),
            DataType::Set(_) => "set".to_string(),
            DataType::Zset(_) => "zset".to_string(),
            DataType::Hash(_) => "hash".to_string(),
            DataType::Stream(_) => "stream".to_string(),
            DataType::Vectorset(_) => "vectorset".to_string(),
        }
    }
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

    pub fn append_to_list(&mut self, value: Vec<String>) -> Result<usize> {
        match &mut self.value {
            DataType::List(existing_list) => {
                existing_list.extend(value);
                Ok(existing_list.len())
            }
            DataType::String(_) => Err(anyhow!("Trying to append to a string datatype")),
            _ => Err(anyhow!("Got a DataType that isn't implemented yet")),
        }
    }

    pub fn prepend_to_list(&mut self, value: Vec<String>) -> Result<usize> {
        match &mut self.value {
            DataType::List(existing_list) => {
                for ele in value {
                    existing_list.insert(0, ele.clone())
                }
                Ok(existing_list.len())
            }
            DataType::String(_) => Err(anyhow!("Trying to append to a string datatype")),
            _ => Err(anyhow!("Got a DataType that isn't implemented yet")),
        }
    }

    pub fn index_list(&self, start: i64, stop: i64) -> Result<&[String]> {
        match &self.value {
            DataType::List(existing_list) => {
                // Account for negative indices
                let start = if start < 0 {
                    existing_list.len().saturating_add_signed(start as isize)
                } else {
                    start as usize
                };
                let stop = if stop < 0 {
                    existing_list.len().saturating_add_signed(stop as isize)
                } else {
                    stop as usize
                };

                // If start >= stop
                if start >= stop {
                    return Ok(&[]); // return empty array;
                }

                // If start larger then list
                if start >= existing_list.len() {
                    return Ok(&[]); // return empty array;
                }
                if stop >= existing_list.len() {
                    return Ok(&existing_list[start..existing_list.len()]);
                }
                // Now slice
                Ok(&existing_list[start..=stop])
            }
            DataType::String(_) => Err(anyhow!("Trying to index a string datatype")),
            _ => Err(anyhow!("Got a DataType that isn't implemented yet")),
        }
    }

    pub fn get_list_len(&self) -> Result<usize> {
        match &self.value {
            DataType::List(existing_list) => Ok(existing_list.len()),
            DataType::String(_) => Err(anyhow!("LLEN key provided is for a string value")),
            _ => Err(anyhow!("Got a DataType that isn't implemented yet")),
        }
    }

    pub fn lpop_list(&mut self, amount: usize) -> Result<Vec<String>> {
        match &mut self.value {
            DataType::List(existing_list) => {
                if existing_list.is_empty() {
                    return Ok(vec![]);
                }
                let popped = existing_list.drain(0..amount).collect();
                Ok(popped)
            }
            DataType::String(_) => Err(anyhow!("LPOP key provided is for a string value")),
            _ => Err(anyhow!("Got a DataType that isn't implemented yet")),
        }
    }
}

pub fn retrieve_from_cache(
    cache: &mut MutexGuard<'_, HashMap<String, RedisValue>>,
    key: &str,
) -> Option<RedisValue> {
    if let Some(value) = cache.get(key) {
        // Check for expiration
        if let Some(expiration) = value.expiration
            && !Utc::now().le(&expiration)
        {
            // Remove expired value
            cache.remove(key);
            None
        } else {
            Some(value.clone())
        }
    } else {
        None
    }
}
