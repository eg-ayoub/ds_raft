use serde::{Serialize, Deserialize};
use mpi::topology::Rank;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RPCType {
    AppendEntries,
    RequestVote
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct RPCParameters {
    pub rv_term: u64,
    pub rv_candidate_id: i32,
    pub rv_last_log_index: usize,
    pub rv_last_log_term: u64
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct RPCMessage {
    pub call_type: RPCType,
    pub call_parameters: RPCParameters
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct RPCResponseParameters {
    pub rv_term: u64,
    pub rv_votegranted: bool
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct RPCResponse {
    pub call_type: RPCType,
    pub response_parameters: RPCResponseParameters,
    pub to: Rank
}