use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Operation {
    Incr,
    Decr,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct LogEntry {
    pub operation: Operation,
    pub term: u64,
}
