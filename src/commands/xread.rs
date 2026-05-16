use crate::{
    RedisCache,
    commands::xadd::EntryId,
    database::cache::{RedisValue, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};

pub fn handle_xread(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    let Some(_) = data.params_list.get(1) else {
        write_buffer.push_str("-ERR missing the STREAMS\r\n");
        return;
    };

    let mut keys = vec![];
    let mut ids = vec![];

    for i in 2..data.params_list.len() {
        let value = data.params_list.get(i).expect("Within params_list range");
        if i < (data.params_list.len() / 2) + 1 {
            keys.push(value.param_value.clone());
        } else {
            let value_string: Option<String> = if value.param_value == "-" {
                None
            } else if value.param_value.split_once("-").is_some() {
                Some(value.param_value.clone())
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
            ids.push(value_entry_id);
        }
    }
    assert_eq!(keys.len(), ids.len());

    write_buffer.push_str(&format!("*{}\r\n", keys.len()));
    for (key, value) in keys.iter().zip(ids) {
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
            write_buffer.push_str("*2\r\n");
            write_buffer.push_str(&format!("${}\r\n{}\r\n", key.len(), key));
            write_buffer.push_str(&xrange_resp);
        } else {
            write_buffer.push_str("-ERR issue reading the range in the stream\r\n");
        }
    }
}
