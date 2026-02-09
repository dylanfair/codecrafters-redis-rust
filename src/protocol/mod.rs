use crate::protocol::parsing::RedisProtocol;

pub mod parsing;

pub fn handle_actions(redis_data: RedisProtocol, write_buffer: &mut String) {
    for param in redis_data.params_list {
        match param.param_value.as_str() {
            "PING" => write_buffer.push_str("+PONG\r\n"),
            _ => {}
        }
    }
}
