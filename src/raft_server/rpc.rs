use serde::{Serialize, Deserialize};
use mpi::topology::Rank;

pub use crate::raft_server::log_entry;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RPCType {
    AppendEntries,
    RequestVote
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestVoteParameters {
    pub term: u64,
    pub candidate_id: i32,
    pub last_log_index: usize,
    pub last_log_term: u64
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppendEntriesParameters {
    pub term: u64,
    pub leader_id: i32,
    pub prev_log_index: usize,
    pub prev_log_term: u64,
    pub entries: Vec<log_entry::LogEntry>,
    pub leader_commit: usize
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RPCMessage {
    pub message: Request,
    pub rtype: RPCType,
    pub rv_params: Option<RequestVoteParameters>,
    pub ae_params: Option<AppendEntriesParameters>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RPCResponseParameters {
    pub term: u64,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RPCResponse {
    pub response: Response,
    pub rtype: RPCType,
    pub params: RPCResponseParameters,
    pub to: Rank
}