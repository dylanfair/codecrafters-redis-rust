use crate::{
    database::cache::{DataType, RedisCache, RedisValue, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};

pub enum PushType {
    Rpush,
    Lpush,
}

pub fn handle_xpush(
    data: RedisProtocol,
    write_buffer: &mut String,
    cache: &RedisCache,
    push: PushType,
) {
    // Get our key
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("- missing the RPUSH key\r\n");
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
        if let Some(mut get_value) = retrieve_from_cache(&mut cache, &some_key.param_value) {
            let push_result = match push {
                PushType::Rpush => get_value.append_to_list(new_elements),
                PushType::Lpush => get_value.prepend_to_list(new_elements),
            };
            match push_result {
                Ok(new_list_size) => {
                    cache.insert(some_key.param_value.to_string(), get_value);
                    write_buffer.push_str(&format!(":{}\r\n", new_list_size));
                }
                Err(e) => write_buffer.push_str(&format!("-{}\r\n", e)),
            }
        } else {
            // If not, make a new list
            let list_len = new_elements.len();
            let redis_value = match push {
                PushType::Rpush => RedisValue::new(DataType::List(new_elements), None),
                PushType::Lpush => {
                    new_elements.reverse();
                    RedisValue::new(DataType::List(new_elements), None)
                }
            };
            cache.insert(some_key.param_value.to_string(), redis_value);
            write_buffer.push_str(&format!(":{}\r\n", list_len));
        }
    } else {
        write_buffer.push_str("-could not get lock to database\r\n");
    }
}
