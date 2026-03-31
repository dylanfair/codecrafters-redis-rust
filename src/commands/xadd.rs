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
            let milliseconds_parsed = split.0.parse::<u64>();
            let sequence_parsed = split.1.parse::<u64>();

            if milliseconds_parsed.is_err() || sequence_parsed.is_err() {
                return Err(
                    "ERR Entry ID must be in the format of <milliseconds>-<sequence number>",
                );
            }

            let milliseconds = milliseconds_parsed.expect("Milliseconds already checked");
            let sequence = sequence_parsed.expect("Secquence already checked");

            if milliseconds == 0 && sequence == 0 {
                return Err("ERR The ID specified in XADD must be greater than 0-0");
            }

            Ok(EntryId::new(milliseconds, sequence))
        } else {
            Err("ERR Entry ID must be in the format of <milliseconds>-<sequence number>")
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
    let mut new_id_string = some_stream_key_value.param_value.to_string();
    if check_id_validity && let Ok(Some(id)) = stream_obj.get_latest_stream_id() {
        let latest_id = EntryId::try_from(id).expect("Came from existing stream");

        if let Some(split) = new_id_string.split_once("-") {
            if split.1 == "*" && split.0 == latest_id.milliseconds_time.to_string() {
                new_id_string = format!("{}-{}", split.0, latest_id.sequence_number + 1);
            } else {
                new_id_string = format!("{}-{}", split.0, 0);
            }
        } else if new_id_string == "*" {
            todo!()
        }

        match EntryId::try_from(new_id_string.clone()) {
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
        // Check for *
        if let Some(split) = new_id_string.split_once("-") {
            if split.1 == "*" {
                if split.0 == "0" {
                    new_id_string = format!("{}-{}", split.0, "1");
                } else {
                    new_id_string = format!("{}-{}", split.0, "0");
                }
            }
        } else if new_id_string == "*" {
            todo!();
        }

        // Check if try from fails (can catch 0-0)
        match EntryId::try_from(new_id_string.clone()) {
            Ok(_) => {}
            Err(e) => {
                write_buffer.push_str(&format!("-{}\r\n", e));
                return;
            }
        }
    }

    // After validation
    let mut new_entry = HashMap::new();
    new_entry.insert("id".to_string(), new_id_string.clone());

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

    match stream_obj.stream_insert(&new_id_string, new_entry) {
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
            some_stream_key_value.param_size, new_id_string
        ));
    } else {
        write_buffer.push_str("- could not get lock to database\r\n");
    }
}
