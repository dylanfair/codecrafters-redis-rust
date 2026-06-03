use std::io::Write;
use std::{net::TcpStream, sync::Arc};

use crate::send_response;
use crate::{protocol::parsing::RedisProtocol, server::server::RedisServer};

pub fn handle_psync(
    stream: &mut TcpStream,
    data: RedisProtocol,
    write_buffer: &mut String,
    server: &Arc<RedisServer>,
) {
    let Some(replication_id) = data.params_list.get(1) else {
        write_buffer.push_str("-missing the replication_id\r\n");
        return;
    };

    let Some(offset) = data.params_list.get(2) else {
        write_buffer.push_str("-missing the offset\r\n");
        return;
    };

    if replication_id.param_value == "?" && offset.param_value == "-1" {
        write_buffer.push_str(&format!(
            "+FULLRESYNC {} 0\r\n",
            &server.master_repl.as_ref().unwrap().master_replid
        ));
        send_response(stream, write_buffer);
        write_buffer.clear();

        // Open a new connection and send?
        let stream_addr = stream.local_addr().unwrap().to_string();
        let mut connection = TcpStream::connect(stream_addr)
            .expect("Failed to open a connection to send the empty RBD file");

        // send RDB transfer
        let bytes = hex::decode("524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2").expect("Failed to decode empty rdb hex");
        write_buffer.push_str(&format!("${}\r\n", bytes.len()));

        if let Err(e) = connection.write(write_buffer.as_bytes()) {
            eprintln!("Failed to write response: {}", e);
        }
        if let Err(e) = connection.write_all(&bytes[..]) {
            eprintln!("Failed to write response: {}", e);
        }
        if let Err(e) = connection.flush() {
            eprintln!("Failed to flush stream: {}", e);
        }
    }
}
