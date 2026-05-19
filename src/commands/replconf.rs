use std::sync::Arc;

use crate::{protocol::parsing::RedisProtocol, server::server::RedisServer};

pub fn handle_replconf(data: RedisProtocol, write_buffer: &mut String, _server: &Arc<RedisServer>) {
    for i in 0..data.params_n {
        let value = data.params_list.get(i).expect("Within params_list range ");
        match value.param_value.to_lowercase().as_str() {
            "listening-port" => {}
            "capa" => {}
            _ => {}
        }
    }
    write_buffer.push_str("+OK\r\n");
}
