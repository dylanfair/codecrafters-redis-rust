use crate::{
    database::cache::{RedisCache, retrieve_from_cache},
    protocol::{parsing::RedisProtocol, writing::resp_encode_array},
};

pub fn handle_lpop(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    // Get our key
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("-missing the LPOP key\r\n");
        return;
    };

    let pop_amount = match data.params_list.get(2) {
        Some(param) => match param.param_value.parse::<usize>() {
            Ok(amount) => amount,
            Err(e) => {
                write_buffer.push_str(&format!(
                    "-could not parse parameter into a usize value: {}\r\n",
                    e
                ));
                return;
            }
        },
        None => 1,
    };

    if let Ok(mut cache) = cache.lock() {
        if let Some(mut get_value) = retrieve_from_cache(&mut cache, &some_key.param_value) {
            match get_value.lpop_list(pop_amount) {
                Ok(popped) => {
                    cache.insert(some_key.param_value.to_string(), get_value);
                    if popped.len() == 1 {
                        let ele = popped.first().unwrap();
                        write_buffer.push_str(&format!("${}\r\n", ele.len()));
                        write_buffer.push_str(&format!("{}\r\n", ele));
                    } else {
                        resp_encode_array(&popped, write_buffer);
                    }
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
