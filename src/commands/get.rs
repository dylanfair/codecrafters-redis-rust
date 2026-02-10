use chrono::Utc;

use crate::{RedisCache, database::cache::DataType, protocol::parsing::RedisProtocol};

pub fn handle_get(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    if let Some(to_get) = data.params_list.get(1) {
        if let Ok(mut cache) = cache.lock() {
            if let Some(get_value) = cache.get(&to_get.param_value) {
                // Check for expiration
                if let Some(expiration) = get_value.expiration
                    && !Utc::now().le(&expiration)
                {
                    // Remove expired value and return nil
                    cache.remove(&to_get.param_value);
                    write_buffer.push_str("$-1\r\n");
                    return;
                }

                match &get_value.value {
                    DataType::String(value) => {
                        let response = format!("${}\r\n{}\r\n", value.len(), value);
                        write_buffer.push_str(&response);
                    }
                    DataType::List(_list) => {
                        todo!();
                    }
                }
            } else {
                write_buffer.push_str("$-1\r\n");
            }
        } else {
            write_buffer.push_str("-ERR could not get lock to database\r\n");
        }
    } else {
        write_buffer.push_str("-ERR no key to GET\r\n");
    }
}
