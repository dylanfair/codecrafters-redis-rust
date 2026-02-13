use crate::{
    RedisCache,
    database::cache::{DataType, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};

pub fn handle_get(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    if let Some(to_get) = data.params_list.get(1) {
        if let Ok(mut cache) = cache.lock() {
            if let Some(get_value) = retrieve_from_cache(&mut cache, &to_get.param_value) {
                match &get_value.value {
                    DataType::String(value) => {
                        let response = format!("${}\r\n{}\r\n", value.len(), value);
                        write_buffer.push_str(&response);
                    }
                    DataType::List(_list) => {
                        todo!();
                    }
                    DataType::Set(_set) => {
                        todo!();
                    }
                    DataType::Zset(_zset) => {
                        todo!();
                    }
                    DataType::Hash(_hash) => {
                        todo!();
                    }
                    DataType::Stream(_stream) => {
                        todo!();
                    }
                    DataType::Vectorset(_vectorset) => {
                        todo!();
                    }
                }
            } else {
                write_buffer.push_str("$-1\r\n");
            }
        } else {
            write_buffer.push_str("-could not get lock to database\r\n");
        }
    } else {
        write_buffer.push_str("-no key to GET\r\n");
    }
}
