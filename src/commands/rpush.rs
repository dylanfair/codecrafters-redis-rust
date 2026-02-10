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
                cache.insert(some_key.param_value.to_string(), redis_value);
                write_buffer.push_str(":1\r\n");
            } else {
                match get_value.append_to_list(some_value.param_value.clone()) {
                    Ok(new_list_size) => {
                        write_buffer.push_str(&format!(":{}\r\n", new_list_size));
                    }
                    Err(e) => write_buffer.push_str(&format!("-ERR {}\r\n", e)),
                }
            }
        } else {
            // If not, make a new list
            let redis_value = RedisValue::new(
                DataType::List(vec![some_value.param_value.to_string()]),
                None,
            );
            cache.insert(some_key.param_value.to_string(), redis_value);
            write_buffer.push_str(":1\r\n");
        }
    } else {
        write_buffer.push_str("-ERR could not get lock to database\r\n");
    }
}
