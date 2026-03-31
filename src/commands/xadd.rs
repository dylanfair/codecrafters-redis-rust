use std::collections::HashMap;

use crate::{
    RedisCache,
    database::cache::{DataType, RedisValue, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};

pub fn handle_xadd(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    let Some(some_stream_key) = data.params_list.get(1) else {
        write_buffer.push_str("- missing the stream key\r\n");
        return;
    };

    let Some(some_stream_key_value) = data.params_list.get(2) else {
        write_buffer.push_str("- missing the stream id\r\n");
        return;
    };

    let mut stream_obj: RedisValue;
    if let Ok(mut cache) = cache.lock() {
        if let Some(existing_stream) = retrieve_from_cache(&mut cache, &some_stream_key.param_value)
        {
            stream_obj = existing_stream;
        } else {
            let mut new_stream = HashMap::new();
            new_stream.insert(
                "id".to_string(),
                some_stream_key_value.param_value.to_string(),
            );
            stream_obj = RedisValue::new(DataType::Stream(new_stream), None);
        };
    } else {
        write_buffer.push_str("- could not get lock to database\r\n");
        return;
    }

    let mut counter = 3;
    loop {
        let Some(key) = data.params_list.get(counter) else {
            break;
        };
        let Some(value) = data.params_list.get(counter + 1) else {
            break;
        };

        match stream_obj.stream_insert(&key.param_value, &value.param_value) {
            Ok(_) => {}
            Err(e) => write_buffer.push_str(&format!("-{}\r\n", e)),
        }
        counter += 2
    }

    if let Ok(mut cache) = cache.lock() {
        cache.insert(some_stream_key.param_value.to_string(), stream_obj);
        write_buffer.push_str(&format!("+{}\r\n", some_stream_key_value.param_value));
    } else {
        write_buffer.push_str("- could not get lock to database\r\n");
    }
}
