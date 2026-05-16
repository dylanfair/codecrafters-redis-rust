use crate::{protocol::parsing::RedisProtocol, server::server::RedisServer};
use std::sync::Arc;

pub fn handle_info(data: RedisProtocol, write_buffer: &mut String, server: &Arc<RedisServer>) {
    for i in 0..data.params_n {
        let value = data.params_list.get(i).expect("Iterating among ");
        if value.param_value.to_uppercase() == "replication" {
            write_buffer.push_str(&format!("$11\r\nrole:{}\r\n", server.role));
        }
    }
}
