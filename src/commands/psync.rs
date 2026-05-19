use std::sync::Arc;

use crate::{protocol::parsing::RedisProtocol, server::server::RedisServer};

pub fn handle_psync(write_buffer: &mut String, server: &Arc<RedisServer>) {
    write_buffer.push_str("+FULLRESYNC");
    write_buffer.push_str(&server.master_repl.as_ref().unwrap().master_replid);
    write_buffer.push_str("0\r\n");
}
