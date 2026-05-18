#[derive(Clone)]
pub struct MasterRepl {
    pub master_replid: String,
    pub master_repl_offset: usize,
}

impl MasterRepl {
    pub fn new(master_replid: String, master_repl_offset: usize) -> Self {
        MasterRepl {
            master_replid,
            master_repl_offset,
        }
    }
}
