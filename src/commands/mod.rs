use crate::protocol::parsing::RedisProtocol;

pub fn handle_echo(protocol: RedisProtocol, write_buffer: &mut String) {
    // respond with $3\r\nhey\r\n
    if let Some(to_be_echoed) = protocol.params_list.get(1) {
        let response = format!(
            "${}\r\n{}\r\n",
            to_be_echoed.param_size, to_be_echoed.param_value
        );
        write_buffer.push_str(&response);
    } else {
        write_buffer.push_str("-ERR no response to echo\r\n");
    }
}
