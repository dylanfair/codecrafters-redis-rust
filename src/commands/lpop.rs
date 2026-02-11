use crate::{
    database::cache::{RedisCache, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};

pub fn handle_lpop(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    // Get our key
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("-missing the LPOP key\r\n");
        return;
    };

    if let Ok(mut cache) = cache.lock() {
        if let Some(mut get_value) = retrieve_from_cache(&mut cache, &some_key.param_value) {
            match get_value.lpop_list() {
                Ok(popped_value) => {
                    cache.insert(some_key.param_value.to_string(), get_value);
                    write_buffer.push_str(&format!("${}\r\n", popped_value.len()));
                    write_buffer.push_str(&format!("{}\r\n", popped_value));
                }
                Err(e) => write_buffer.push_str(&format!("-{}\r\n", e)),
            }
        } else {
            write_buffer.push_str(":0\r\n");
        }
    } else {
        write_buffer.push_str("-could not get lock to database\r\n");
    }
}
