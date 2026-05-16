use crate::protocol::parsing::RedisProtocol;

pub fn handle_info(_data: RedisProtocol, write_buffer: &mut String) {
    write_buffer.push_str("$11\r\nrole:master\r\n");
}
