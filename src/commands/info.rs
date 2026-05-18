use crate::{
    protocol::parsing::RedisProtocol,
    server::server::{RedisRole, RedisServer},
};
use std::sync::Arc;

pub fn handle_info(data: RedisProtocol, write_buffer: &mut String, server: &Arc<RedisServer>) {
    for i in 0..data.params_n {
        let value = data.params_list.get(i).expect("Within params_list range ");
        if value.param_value.to_uppercase() == "REPLICATION" {
            let role = format!("role:{}", server.role);
            write_buffer.push_str(&format!("${}\r\n{}\r\n", role.len(), role));
            match server.role {
                RedisRole::Master => {
                    let master_replid = format!(
                        "master_replid:{}",
                        server.master_repl.as_ref().unwrap().master_replid
                    );
                    let master_repl_offset = format!(
                        "master_repl_offset:{}",
                        server.master_repl.as_ref().unwrap().master_repl_offset
                    );
                    write_buffer.push_str(&format!(
                        "${}\r\n{}\r\n",
                        master_replid.len(),
                        master_replid
                    ));
                    write_buffer.push_str(&format!(
                        "${}\r\n{}\r\n",
                        master_repl_offset.len(),
                        master_repl_offset
                    ));
                }
                RedisRole::Slave => {}
            }
        }
    }
}
