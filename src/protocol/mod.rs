use crate::{commands::handle_echo, protocol::parsing::RedisProtocol};

pub mod parsing;

pub fn handle_commands(redis_data: RedisProtocol, write_buffer: &mut String) {
    if let Some(action) = redis_data.params_list.first() {
        match action.param_value.to_lowercase().as_str() {
            "ping" => write_buffer.push_str("+PONG\r\n"),
            "echo" => handle_echo(redis_data, write_buffer),
            _ => {}
        }
    } else {
        write_buffer.push_str("-ERR no command to run\r\n")
    }
}
