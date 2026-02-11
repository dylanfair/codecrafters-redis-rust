use crate::{
    database::cache::{RedisCache, retrieve_from_cache},
    protocol::{parsing::RedisProtocol, writing::resp_encode_array},
};
use chrono::{Duration, Utc};

pub fn handle_blpop(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    // Get our key
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("-missing the BLPOP key\r\n");
        return;
    };

    let timeout = match data.params_list.get(2) {
        Some(param) => match param.param_value.parse::<f64>() {
            Ok(amount) => amount,
            Err(e) => {
                write_buffer.push_str(&format!(
                    "-could not parse parameter into a usize value: {}\r\n",
                    e
                ));
                return;
            }
        },
        None => 0.0,
    };
    let blocking_expiration = if timeout > 0.0 {
        let milis = (timeout * 1000.0) as i64;
        let expiration = Utc::now() + Duration::milliseconds(milis);
        Some(expiration)
    } else {
        None
    };

    loop {
        if let Ok(mut cache) = cache.lock() {
            if let Some(mut get_value) = retrieve_from_cache(&mut cache, &some_key.param_value) {
                match get_value.lpop_list(1) {
                    Ok(popped) => {
                        cache.insert(some_key.param_value.to_string(), get_value);
                        let blpop_vec = vec![
                            some_key.param_value.to_string(),
                            popped.first().expect("Wouldn't pop otherwise").to_string(),
                        ];
                        resp_encode_array(&blpop_vec, write_buffer);
                        return;
                    }
                    Err(e) => {
                        write_buffer.push_str(&format!("-{}\r\n", e));
                        return;
                    }
                }
            }
            if let Some(expiration) = blocking_expiration
                && expiration <= Utc::now()
            {
                break;
            }
        } else {
            write_buffer.push_str("-could not get lock to database\r\n");
            return;
        }
    }
    write_buffer.push_str("*-1\r\n");
}
