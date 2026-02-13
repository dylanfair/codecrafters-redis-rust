use crate::{
    database::cache::{RedisCache, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};

pub fn handle_type(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    if let Some(to_get) = data.params_list.get(1) {
        if let Ok(mut cache) = cache.lock() {
            if let Some(get_value) = retrieve_from_cache(&mut cache, &to_get.param_value) {
                write_buffer.push_str(&format!("+{}\r\n", get_value.datatype_str()));
            } else {
                write_buffer.push_str("+none\r\n");
            }
        } else {
            write_buffer.push_str("-could not get lock to database\r\n");
        }
    } else {
        write_buffer.push_str("-no key to GET\r\n");
    }
}
