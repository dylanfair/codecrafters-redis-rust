use anyhow::{Result, anyhow};

#[derive(Clone)]
pub struct ReplicaInfo {
    location: String,
    port: String,
}

impl ReplicaInfo {
    pub fn new(location: String, port: String) -> Self {
        ReplicaInfo { location, port }
    }

    pub fn arg_parse(raw_arg: String) -> Result<Self> {
        match raw_arg.split_once(" ") {
            Some((location, port)) => Ok(ReplicaInfo::new(location.to_string(), port.to_string())),
            None => Err(anyhow!(
                "Could not parse the 'replicaof' argument properly. Please entry as '<location> <port>'"
            )),
        }
    }
}
