use crate::{
    database::cache::{RedisCache, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};
use anyhow::{Result, anyhow};

pub fn handle_lrange(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("-missing the SET key\r\n");
        return;
    };
    match handle_params(&data) {
        Ok((start, stop)) => {
            if start >= stop {
                write_buffer.push_str("*0\r\n");
                return;
            }

            if let Ok(mut cache) = cache.lock() {
                if let Some(get_value) = retrieve_from_cache(&mut cache, &some_key.param_value) {
                    match get_value.index_list(start, stop) {
                        Ok(list_slice) => {
                            if list_slice.is_empty() {
                                write_buffer.push_str("*0\r\n");
                            } else {
                                write_buffer.push_str(&format!("*{}\r\n", list_slice.len()));
                                for element in list_slice {
                                    write_buffer.push_str(&format!("${}\r\n", element.len()));
                                    write_buffer.push_str(&format!("{}\r\n", element));
                                }
                            }
                        }
                        Err(e) => write_buffer.push_str(&format!("-{}\r\n", e)),
                    }
                } else {
                    write_buffer.push_str("*0\r\n");
                }
            } else {
                write_buffer.push_str("-could not get lock to database\r\n");
            }
        }
        Err(e) => write_buffer.push_str(&format!("-{}\r\n", e)),
    }
}

fn handle_params(data: &RedisProtocol) -> Result<(usize, usize)> {
    let Some(start_param) = data.params_list.get(1) else {
        return Err(anyhow!("missing START"));
    };
    let Some(stop_param) = data.params_list.get(2) else {
        return Err(anyhow!("missing STOP"));
    };
    let Ok(start) = start_param.param_value.parse::<usize>() else {
        return Err(anyhow!("START must be a valid number"));
    };
    let Ok(stop) = stop_param.param_value.parse::<usize>() else {
        return Err(anyhow!("STOP must be a valid number"));
    };
    Ok((start, stop))
}
