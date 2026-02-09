use crate::{RedisCache, protocol::parsing::RedisProtocol};

pub fn handle_get(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    if let Some(to_get) = data.params_list.get(1) {
        if let Ok(cache) = cache.lock() {
            if let Some(get_value) = cache.get(&to_get.param_value) {
                let response = format!("${}\r\n{}\r\n", get_value.len(), get_value);
                write_buffer.push_str(&response);
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
