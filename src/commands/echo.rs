use crate::protocol::parsing::RedisProtocol;

pub fn handle_echo(data: RedisProtocol, write_buffer: &mut String) {
    if let Some(to_be_echoed) = data.params_list.get(1) {
        let response = format!(
            "${}\r\n{}\r\n",
            to_be_echoed.param_size, to_be_echoed.param_value
        );
        write_buffer.push_str(&response);
    } else {
        write_buffer.push_str("-no response to ECHO\r\n");
    }
}
