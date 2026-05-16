use crate::{
    RedisCache,
    commands::xadd::EntryId,
    database::cache::{RedisValue, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};
use chrono::{Duration, Utc};

struct Block {
    pub block: bool,
    pub timeout: u64,
}

impl Block {
    fn new(block: bool, timeout: u64) -> Self {
        Block { block, timeout }
    }
}

struct StreamParams {
    pub keys: Vec<String>,
    pub ids: Vec<EntryId>,
}

impl StreamParams {
    fn new(keys: Vec<String>, ids: Vec<EntryId>) -> Self {
        StreamParams { keys, ids }
    }
}

struct XReadInterface {
    pub block: Block,
    pub stream: StreamParams,
}

impl XReadInterface {
    fn default() -> Self {
        XReadInterface {
            block: Block::new(false, 0),
            stream: StreamParams::new(vec![], vec![]),
        }
    }
}

pub fn handle_xread(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    let mut xread_params = XReadInterface::default();
    let mut stream_start = 2;

    for i in 1..data.params_list.len() {
        let param = data
            .params_list
            .get(i)
            .expect("Within frist param_list range");

        if param.param_value.to_uppercase() == "BLOCK" {
            xread_params.block.block = true;
            let timeout = data.params_list.get(i + 1);
            match timeout {
                Some(timeout) => match timeout.param_value.parse::<u64>() {
                    Ok(timeout_u64) => xread_params.block.timeout = timeout_u64,
                    Err(_) => {
                        write_buffer.push_str(
                            "-ERR timeout with the BLOCK param needs to be a u64 number\r\n",
                        );
                        return;
                    }
                },
                None => {
                    write_buffer.push_str("-ERR missing a timeout with the BLOCK param\r\n");
                    return;
                }
            }
        }
        if param.param_value.to_uppercase() == "STREAMS" {
            stream_start = i + 1;
        }
    }

    let keys_and_ids_len = data.params_list.len() - stream_start;
    let streams_midpoint = (data.params_list.len() - keys_and_ids_len) + (keys_and_ids_len / 2);

    for i in stream_start..data.params_list.len() {
        let value = data.params_list.get(i).expect("Within params_list range");

        if i < streams_midpoint {
            // before midpoint is a key
            xread_params.stream.keys.push(value.param_value.clone());
        } else {
            // after midpoint is an entry ID
            let value_string: Option<String> = if value.param_value == "-" {
                None
            } else if value.param_value.split_once("-").is_some() {
                Some(value.param_value.clone())
            } else if value.param_value == "$" {
                todo!();
            } else {
                Some(format!("{}-0", value.param_value))
            };

            let value_entry_id: EntryId = if let Some(value_string) = value_string {
                match EntryId::try_from(value_string) {
                    Ok(new_id) => new_id,
                    Err(e) => {
                        write_buffer.push_str(&format!("-{}\r\n", e));
                        return;
                    }
                }
            } else {
                write_buffer.push_str("-ERR missing the valid id not sent\r\n");
                return;
            };
            xread_params.stream.ids.push(value_entry_id);
        }
    }
    assert_eq!(
        xread_params.stream.keys.len(),
        xread_params.stream.ids.len()
    );

    let blocking_expiration = if xread_params.block.timeout > 0 {
        let expiration = Utc::now() + Duration::milliseconds(xread_params.block.timeout as i64);
        Some(expiration)
    } else {
        None
    };

    let mut successful_resp = String::from(&format!("*{}\r\n", xread_params.stream.keys.len()));
    let mut any_success = false;
    loop {
        for (key, value) in xread_params
            .stream
            .keys
            .iter()
            .zip(&xread_params.stream.ids)
        {
            // Grab latest stream if it exists
            let stream_obj: RedisValue;
            if let Ok(mut cache) = cache.lock() {
                if let Some(existing_stream) = retrieve_from_cache(&mut cache, key) {
                    stream_obj = existing_stream;
                } else {
                    write_buffer.push_str("*0\r\n");
                    return;
                };
            } else {
                write_buffer.push_str("-ERR could not get lock to database\r\n");
                return;
            }
            // Get range of values
            if let Ok(xrange_resp) = stream_obj.stream_xread(value) {
                if xrange_resp.is_empty() {
                    continue;
                }

                any_success = true;
                successful_resp.push_str("*2\r\n");
                successful_resp.push_str(&format!("${}\r\n{}\r\n", key.len(), key));
                successful_resp.push_str(&xrange_resp);
            } else {
                write_buffer.push_str("-ERR issue reading the range in the stream\r\n");
                return;
            }
        }

        // If we aren't blocking, just exit
        if !xread_params.block.block {
            if !any_success {
                successful_resp = String::from("*-1\r\n");
            }
            break;
        }

        if let Some(expiration) = blocking_expiration
            && expiration <= Utc::now()
        {
            if !any_success {
                successful_resp = String::from("*-1\r\n");
            }
            break;
        }
    }

    write_buffer.push_str(&successful_resp);
}
