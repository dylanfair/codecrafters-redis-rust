use crate::database::cache::{DataType, ExpirationFidelity, RedisValue};
use crate::protocol::parsing::RedisParam;
use crate::{RedisCache, protocol::parsing::RedisProtocol};

#[derive(Default)]
struct SetOptions {
    nx: bool,
    xx: bool,
    ifeq: Option<String>,
    ifne: Option<String>,
    ifdeq: Option<String>,
    ifdne: Option<String>,
    get: bool,
    ex: Option<u64>,
    px: Option<u64>,
    exat: Option<u64>,
    pxat: Option<u64>,
    keepttl: bool,
}

pub fn handle_set(data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    let Some(some_key) = data.params_list.get(1) else {
        write_buffer.push_str("- missing the SET key\r\n");
        return;
    };
    let Some(some_value) = data.params_list.get(2) else {
        write_buffer.push_str("- missing the SET value\r\n");
        return;
    };

    // parse options
    let mut set_options = SetOptions::default();
    if data.params_n > 2 {
        let mut skip = false;
        for i in 3..data.params_n {
            if skip {
                skip = false;
                continue;
            }

            let param_value = &data.params_list[i].param_value;
            match param_value.to_lowercase().as_str() {
                "nx" => set_options.nx = true,
                "xx" => set_options.xx = true,
                "ifeq" => {
                    if let Some(next) = data.params_list.get(i + 1) {
                        set_options.ifeq = Some(next.param_value.to_string());
                        skip = true;
                    } else {
                        write_buffer.push_str("- missing parameter\r\n");
                        return;
                    }
                }
                "ifne" => {
                    if let Some(next) = data.params_list.get(i + 1) {
                        set_options.ifne = Some(next.param_value.to_string());
                        skip = true;
                    } else {
                        write_buffer.push_str("- missing parameter\r\n");
                        return;
                    }
                }
                "ifdeq" => {
                    if let Some(next) = data.params_list.get(i + 1) {
                        set_options.ifdeq = Some(next.param_value.to_string());
                        skip = true;
                    } else {
                        write_buffer.push_str("- missing parameter\r\n");
                        return;
                    }
                }
                "ifdne" => {
                    if let Some(next) = data.params_list.get(i + 1) {
                        set_options.ifdne = Some(next.param_value.to_string());
                        skip = true;
                    } else {
                        write_buffer.push_str("- missing parameter\r\n");
                        return;
                    }
                }
                "get" => set_options.get = true,
                "ex" => {
                    if let Some(next) = data.params_list.get(i + 1) {
                        if let Ok(expiration_value) = next.param_value.parse::<u64>() {
                            set_options.ex = Some(expiration_value);
                            skip = true;
                        } else {
                            write_buffer
                                .push_str("- failed to parse expiration value given\r\n");
                            return;
                        }
                    } else {
                        write_buffer.push_str("- missing parameter\r\n");
                        return;
                    }
                }
                "px" => {
                    if let Some(next) = data.params_list.get(i + 1) {
                        if let Ok(expiration_value) = next.param_value.parse::<u64>() {
                            set_options.px = Some(expiration_value);
                            skip = true;
                        } else {
                            write_buffer
                                .push_str("- failed to parse expiration value given\r\n");
                            return;
                        }
                    } else {
                        write_buffer.push_str("- missing parameter\r\n");
                        return;
                    }
                }
                "exat" => {
                    if let Some(next) = data.params_list.get(i + 1) {
                        if let Ok(expiration_value) = next.param_value.parse::<u64>() {
                            set_options.exat = Some(expiration_value);
                        } else {
                            write_buffer
                                .push_str("- failed to parse expiration timestamp given\r\n");
                            return;
                        }
                    } else {
                        write_buffer.push_str("- missing parameter\r\n");
                        return;
                    }
                    skip = true;
                }
                "pxat" => {
                    if let Some(next) = data.params_list.get(i + 1) {
                        if let Ok(expiration_value) = next.param_value.parse::<u64>() {
                            set_options.pxat = Some(expiration_value);
                        } else {
                            write_buffer
                                .push_str("- failed to parse expiration timestamp given\r\n");
                            return;
                        }
                    } else {
                        write_buffer.push_str("- missing parameter\r\n");
                        return;
                    }
                    skip = true;
                }
                "keepttl" => set_options.keepttl = true,
                _ => {}
            }
        }
    }

    if let Ok(mut cache) = cache.lock() {
        let redis_value = make_redis_value(some_value, &set_options);
        cache.insert(some_key.param_value.to_string(), redis_value);
        write_buffer.push_str("+OK\r\n");
    } else {
        write_buffer.push_str("- could not get lock to database\r\n");
    }
}

fn make_redis_value(value: &RedisParam, set_options: &SetOptions) -> RedisValue {
    if let Some(expiration_value) = set_options.ex {
        RedisValue::new(
            DataType::String(value.param_value.to_string()),
            Some(ExpirationFidelity::Ex(expiration_value)),
        )
    } else if let Some(expiration_value) = set_options.px {
        RedisValue::new(
            DataType::String(value.param_value.to_string()),
            Some(ExpirationFidelity::Px(expiration_value)),
        )
    } else if let Some(expiration_value) = set_options.exat {
        RedisValue::new(
            DataType::String(value.param_value.to_string()),
            Some(ExpirationFidelity::Exat(expiration_value)),
        )
    } else if let Some(expiration_value) = set_options.pxat {
        RedisValue::new(
            DataType::String(value.param_value.to_string()),
            Some(ExpirationFidelity::Pxat(expiration_value)),
        )
    } else {
        RedisValue::new(DataType::String(value.param_value.to_string()), None)
    }
}
