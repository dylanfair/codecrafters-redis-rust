use crate::{RedisCache, protocol::parsing::RedisProtocol};

pub fn handle_set(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("-ERR missing the SET key\r\n");
        return;
    };
    let Some(some_value) = data.params_list.get(2) else {
        write_buffer.push_str("-ERR missing the SET value\r\n");
        return;
    };

    if let Ok(mut cache) = cache.lock() {
        cache.insert(
            some_key.param_value.to_string(),
            some_value.param_value.to_string(),
        );
        write_buffer.push_str("+OK\r\n");
    } else {
        write_buffer.push_str("-ERR could not get lock to database\r\n");
    }
}
