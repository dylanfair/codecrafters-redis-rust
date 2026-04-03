use crate::{
    RedisCache,
    commands::xadd::EntryId,
    database::cache::{RedisValue, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};

pub fn handle_xrange(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    let Some(some_stream_key) = data.params_list.get(1) else {
        write_buffer.push_str("-ERR missing the stream key\r\n");
        return;
    };

    let Some(some_start) = data.params_list.get(2) else {
        write_buffer.push_str("-ERR missing the stream start\r\n");
        return;
    };

    let Some(some_end) = data.params_list.get(3) else {
        write_buffer.push_str("-ERR missing the stream end\r\n");
        return;
    };

    // Grab latest stream if it exists
    let stream_obj: RedisValue;
    if let Ok(mut cache) = cache.lock() {
        if let Some(existing_stream) = retrieve_from_cache(&mut cache, &some_stream_key.param_value)
        {
            stream_obj = existing_stream;
        } else {
            write_buffer.push_str("*0\r\n");
            return;
        };
    } else {
        write_buffer.push_str("-ERR could not get lock to database\r\n");
        return;
    }

    let start_string: String;
    if let Some(_) = some_start.param_value.split_once("-") {
        start_string = some_start.param_value.clone();
    } else {
        start_string = format!("{}-0", some_start.param_value);
    }

    let end_string: String;
    if let Some(_) = some_end.param_value.split_once("-") {
        end_string = some_end.param_value.clone();
    } else {
        // todo! figure out some way to set a "max"
        end_string = format!("{}-0", some_end.param_value);
    }

    let start_entry_id: EntryId;
    match EntryId::try_from(start_string) {
        Ok(new_id) => {
            start_entry_id = new_id;
        }
        Err(e) => {
            write_buffer.push_str(&format!("-{}\r\n", e));
            return;
        }
    }

    let end_entry_id: EntryId;
    match EntryId::try_from(end_string) {
        Ok(new_id) => {
            end_entry_id = new_id;
        }
        Err(e) => {
            write_buffer.push_str(&format!("-{}\r\n", e));
            return;
        }
    }

    // Get range of values
    if let Ok(xrange_resp) = stream_obj.stream_xrange(start_entry_id, end_entry_id) {
        write_buffer.push_str(&xrange_resp);
        return;
    } else {
        write_buffer.push_str("-ERR issue reading the range in the stream\r\n");
        return;
    }
}
