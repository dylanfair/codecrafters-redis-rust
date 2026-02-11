use crate::{
    database::cache::{RedisCache, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};

pub fn handle_llen(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    // Get our key
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("-missing the LLEN key\r\n");
        return;
    };

    if let Ok(mut cache) = cache.lock() {
        if let Some(get_value) = retrieve_from_cache(&mut cache, &some_key.param_value) {
            match get_value.get_list_len() {
                Ok(list_len) => write_buffer.push_str(&format!(":{}\r\n", list_len)),
                Err(e) => write_buffer.push_str(&format!("-{}\r\n", e)),
            }
        } else {
            write_buffer.push_str(":0\r\n");
        }
    } else {
        write_buffer.push_str("-could not get lock to database\r\n");
    }
}
