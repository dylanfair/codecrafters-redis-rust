use crate::{
    protocol::parsing::RedisProtocol,
    server::server::{RedisRole, RedisServer},
};
use std::sync::Arc;

pub fn handle_info(data: RedisProtocol, write_buffer: &mut String, server: &Arc<RedisServer>) {
    let mut info_response = String::new();

    for i in 0..data.params_n {
        let value = data.params_list.get(i).expect("Within params_list range ");
        if value.param_value.to_uppercase() == "REPLICATION" {
            let role = format!("role:{}", server.role);
            info_response.push_str(&role);
            match server.role {
                RedisRole::Master => {
                    let master_replid = format!(
                        "\r\nmaster_replid:{}",
                        server.master_repl.as_ref().unwrap().master_replid
                    );
                    let master_repl_offset = format!(
                        "\r\nmaster_repl_offset:{}",
                        server.master_repl.as_ref().unwrap().master_repl_offset
                    );

                    info_response.push_str(&master_replid);
                    info_response.push_str(&master_repl_offset);
                }
                RedisRole::Slave => {}
            }
        }
    }

    info_response = format!("${}\r\n{}\r\n", info_response.len(), info_response);
    write_buffer.push_str(&info_response);
}
