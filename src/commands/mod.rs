pub mod echo;
pub mod get;
pub mod lrange;
pub mod rpush;
pub mod set;

use crate::RedisCache;
use crate::commands::lrange::handle_lrange;
use crate::commands::rpush::handle_rpush;
use crate::commands::{echo::handle_echo, get::handle_get, set::handle_set};
use crate::protocol::parsing::RedisProtocol;

pub fn handle_commands(redis_data: RedisProtocol, write_buffer: &mut String, cache: &RedisCache) {
    if let Some(action) = redis_data.params_list.first() {
        match action.param_value.to_lowercase().as_str() {
            "ping" => write_buffer.push_str("+PONG\r\n"),
            "echo" => handle_echo(redis_data, write_buffer),
            "set" => handle_set(redis_data, write_buffer, cache),
            "get" => handle_get(redis_data, write_buffer, cache),
            "rpush" => handle_rpush(redis_data, write_buffer, cache),
            "lrange" => handle_lrange(redis_data, write_buffer, cache),
            _ => {}
        }
    } else {
        write_buffer.push_str("- no command to run\r\n")
    }
}
