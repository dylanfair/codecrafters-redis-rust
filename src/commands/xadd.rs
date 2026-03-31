use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::{cmp::PartialOrd, collections::HashMap};

use crate::{
    RedisCache,
    database::cache::{DataType, RedisValue, retrieve_from_cache},
    protocol::parsing::RedisProtocol,
};

#[derive(PartialEq)]
struct EntryId {
    milliseconds_time: u64,
    sequence_number: u64,
}

impl EntryId {
    fn new(milliseconds_time: u64, sequence_number: u64) -> EntryId {
        EntryId {
            milliseconds_time,
            sequence_number,
        }
    }

    fn greater_than_zero(&self) -> bool {
        !(self.milliseconds_time == 0 && self.sequence_number == 0)
    }
}

impl PartialOrd for EntryId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.milliseconds_time == other.milliseconds_time {
            self.sequence_number.partial_cmp(&other.sequence_number)
        } else {
            self.milliseconds_time.partial_cmp(&other.milliseconds_time)
        }
    }
}

impl TryFrom<String> for EntryId {
    type Error = &'static str;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        if let Some(split) = id.split_once("-") {
            let milliseconds = split.0.parse::<u64>();
            let sequence = split.0.parse::<u64>();

            if milliseconds.is_err() || sequence.is_err() {
                return Err("Entry ID must be in the format of <milliseconds>-<sequence number>");
            }

            Ok(EntryId::new(
                milliseconds.expect("Milliseconds already checked"),
                sequence.expect("Sequence already checked"),
            ))
        } else {
            Err("Entry ID must be in the format of <milliseconds>-<sequence number>")
        }
    }
}

pub fn handle_xadd(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    let Some(some_stream_key) = data.params_list.get(1) else {
        write_buffer.push_str("-ERR missing the stream key\r\n");
        return;
    };

    let Some(some_stream_key_value) = data.params_list.get(2) else {
        write_buffer.push_str("-ERR missing the stream id\r\n");
        return;
    };

    // Grab latest stream if it exists
    let mut stream_obj: RedisValue;
    let mut check_id_validity = false;
    if let Ok(mut cache) = cache.lock() {
        if let Some(existing_stream) = retrieve_from_cache(&mut cache, &some_stream_key.param_value)
        {
            stream_obj = existing_stream;
            check_id_validity = true;
        } else {
            stream_obj = RedisValue::new(DataType::Stream(BTreeMap::new()), None);
        };
    } else {
        write_buffer.push_str("-ERR could not get lock to database\r\n");
        return;
    }

    // Validate ID here
    if check_id_validity && let Ok(Some(id)) = stream_obj.get_latest_stream_id() {
        let latest_id = EntryId::try_from(id).expect("Came from existing stream");
        match EntryId::try_from(some_stream_key_value.param_value.to_string()) {
            Ok(new_id) => {
                // compare here
                if new_id <= latest_id {
                    write_buffer.push_str("-ERR The ID specified in XADD is equal or smaller than the target stream top item\r\n");
                    return;
                }
            }
            Err(e) => {
                write_buffer.push_str(&format!("-{}\r\n", e));
                return;
            }
        }
    } else {
        // we are pushing into an empty stream - check if ID is greater than 0-0
        match EntryId::try_from(some_stream_key_value.param_value.to_string()) {
            Ok(new_id) => {
                if !new_id.greater_than_zero() {
                    write_buffer
                        .push_str("-ERR The ID specified in XADD must be greater than 0-0\r\n");
                    return;
                }
            }
            Err(e) => {
                write_buffer.push_str(&format!("-{}\r\n", e));
                return;
            }
        }
    }

    // After validation
    let mut new_entry = HashMap::new();
    new_entry.insert("id".to_string(), some_stream_key_value.param_value.clone());

    let mut counter = 3;
    loop {
        let Some(key) = data.params_list.get(counter) else {
            break;
        };
        let Some(value) = data.params_list.get(counter + 1) else {
            break;
        };
        new_entry.insert(key.param_value.clone(), value.param_value.clone());

        counter += 2
    }

    match stream_obj.stream_insert(&some_stream_key_value.param_value, new_entry) {
        Ok(_) => {}
        Err(e) => {
            write_buffer.push_str(&format!("-{}\r\n", e));
            return;
        }
    }

    if let Ok(mut cache) = cache.lock() {
        cache.insert(some_stream_key.param_value.to_string(), stream_obj);
        write_buffer.push_str(&format!(
            "${}\r\n{}\r\n",
            some_stream_key_value.param_size, some_stream_key_value.param_value
        ));
    } else {
        write_buffer.push_str("- could not get lock to database\r\n");
    }
}
