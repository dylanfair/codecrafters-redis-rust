use crate::{
    database::cache::{DataType, RedisCache, RedisValue},
    protocol::parsing::RedisProtocol,
};
use chrono::Utc;

pub fn handle_rpush(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    // Get our key
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("-ERR missing the RPUSH key\r\n");
        return;
    };

    // Get our elements
    let mut new_elements = vec![];
    for i in 2..data.params_n {
        let new_value = data
            .params_list
            .get(i)
            .expect("We should always have something with earlier validity check");
        new_elements.push(new_value.param_value.clone());
    }

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
                let list_len = new_elements.len();
                let redis_value = RedisValue::new(DataType::List(new_elements), None);
                cache.insert(some_key.param_value.to_string(), redis_value);
                write_buffer.push_str(&format!(":{}\r\n", list_len));
            } else {
                match get_value.append_to_list(new_elements) {
                    Ok(new_list_size) => {
                        write_buffer.push_str(&format!(":{}\r\n", new_list_size));
                    }
                    Err(e) => write_buffer.push_str(&format!("-ERR {}\r\n", e)),
                }
            }
        } else {
            // If not, make a new list
            let list_len = new_elements.len();
            let redis_value = RedisValue::new(DataType::List(new_elements), None);
            cache.insert(some_key.param_value.to_string(), redis_value);
            write_buffer.push_str(&format!(":{}\r\n", list_len));
        }
    } else {
        write_buffer.push_str("-ERR could not get lock to database\r\n");
    }
}
