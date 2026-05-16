use crate::server::replica::ReplicaInfo;

#[derive(Clone)]
pub struct RedisServer {
    pub role: String,
    pub replicaof: Option<ReplicaInfo>,
}

impl RedisServer {
    pub fn new(role: String, replicaof: Option<ReplicaInfo>) -> Self {
        RedisServer { role, replicaof }
    }
}
