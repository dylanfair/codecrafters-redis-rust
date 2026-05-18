use std::fmt;
use std::str::FromStr;
use strum_macros::EnumString;

use crate::server::master_repl::MasterRepl;
use crate::server::replica::ReplicaInfo;

#[derive(Clone)]
pub struct RedisServer {
    pub role: RedisRole,
    pub replicaof: Option<ReplicaInfo>,
    pub master_repl: Option<MasterRepl>,
}

#[derive(EnumString, Clone)]
pub enum RedisRole {
    #[strum(serialize = "master")]
    Master,
    #[strum(serialize = "slave")]
    Slave,
}

impl fmt::Display for RedisRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedisRole::Master => write!(f, "master"),
            RedisRole::Slave => write!(f, "slave"),
        }
    }
}

impl RedisServer {
    pub fn new(role: String, replicaof: Option<ReplicaInfo>) -> Self {
        let role = RedisRole::from_str(&role).unwrap();
        let master_repl = match role {
            RedisRole::Master => Some(MasterRepl::new(
                "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_string(),
                0,
            )),
            RedisRole::Slave => None,
        };
        RedisServer {
            role,
            replicaof,
            master_repl,
        }
    }
}
