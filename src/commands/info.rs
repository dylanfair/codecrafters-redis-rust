use crate::{
    protocol::parsing::RedisProtocol,
    server::server::{RedisRole, RedisServer},
};
use std::sync::Arc;

pub fn handle_info(data: RedisProtocol, write_buffer: &mut String, server: &Arc<RedisServer>) {
    let mut info_response = String::new();
    let mut lines = 1;

    for i in 0..data.params_n {
        let value = data.params_list.get(i).expect("Within params_list range ");
        if value.param_value.to_uppercase() == "REPLICATION" {
            let role = format!("role:{}", server.role);
            info_response.push_str(&format!("${}\r\n{}\r\n", role.len(), role));
            match server.role {
                RedisRole::Master => {
                    lines += 2;

                    let master_replid = format!(
                        "master_replid:{}",
                        server.master_repl.as_ref().unwrap().master_replid
                    );
                    let master_repl_offset = format!(
                        "master_repl_offset:{}",
                        server.master_repl.as_ref().unwrap().master_repl_offset
                    );

                    info_response.push_str(&format!(
                        "${}\r\n{}\r\n",
                        master_replid.len(),
                        master_replid
                    ));
                    info_response.push_str(&format!(
                        "${}\r\n{}\r\n",
                        master_repl_offset.len(),
                        master_repl_offset
                    ));
                }
                RedisRole::Slave => {}
            }
        }
    }

    info_response = format!("*{lines}\r\n{info_response}");
    println!("{info_response}");
    write_buffer.push_str(&info_response);
}
