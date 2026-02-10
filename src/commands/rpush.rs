use crate::{
    database::cache::{DataType, RedisCache, RedisValue},
    protocol::parsing::RedisProtocol,
};
use chrono::Utc;

pub fn handle_rpush(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("-ERR missing the RPUSH key\r\n");
        return;
    };
    let Some(some_value) = data.params_list.get(2) else {
        write_buffer.push_str("-ERR missing the RPUSH value\r\n");
        return;
    };

    if let Ok(mut cache) = cache.lock() {
        // Make our value
        // First check if there is already a list
        if let Some(get_value) = cache.get_mut(&some_key.param_value) {
            // Check for expiration
            if let Some(expiration) = get_value.expiration
                && !Utc::now().le(&expiration)
            {
                // Remove expired value
                cache.remove(&some_key.param_value);
                // Then add a new list
                let redis_value = RedisValue::new(
                    DataType::List(vec![some_value.param_value.to_string()]),
                    None,
                );
                cache.insert(some_value.param_value.to_string(), redis_value);
                write_buffer.push_str(":1\r\n");
            } else {
                // Otherwise, check if a list
                match &mut get_value.value {
                    DataType::List(vector) => {
                        vector.push(some_value.param_value.clone());
                        // let redis_value = RedisValue::new(DataType::List(vector), None);
                        // cache.insert(some_value.param_value.to_string(), redis_value);
                        write_buffer.push_str(&format!(":{}\r\n", vector.len()));
                    }
                    DataType::String(_) => {
                        write_buffer.push_str("-ERR trying to RPUSH to a non-list");
                    }
                }
            }
        } else {
            // If not, makea new list
            let redis_value = RedisValue::new(
                DataType::List(vec![some_value.param_value.to_string()]),
                None,
            );
            cache.insert(some_value.param_value.to_string(), redis_value);
            write_buffer.push_str(":1\r\n");
        }
    } else {
        write_buffer.push_str("-ERR could not get lock to database\r\n");
    }
}
