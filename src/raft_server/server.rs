use std::collections::HashMap;
use mpi::topology::Rank;
use std::fs::File;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};
use std::io::Result;

pub use crate::raft_server::log_entry;
pub use crate::raft_server::rpc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    Follower,
    Leader,
    Candidate,
}

pub trait PersistentData {
    fn new(rank: Rank) -> Self;
    fn update(&self, rank: Rank) -> Result<()>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RaftPersistence {
    pub current_term: u64,
    pub voted_for: Rank,
    pub log: Vec<log_entry::LogEntry>
}

impl PersistentData for RaftPersistence {
    // * open file if exists and correct
    fn new(rank: Rank) -> RaftPersistence {
        let file_name = format!("server.{}.persistence", rank);
        let file_exists = std::path::Path::new(&file_name).exists();
        if file_exists {
            let mut file = File::open(&file_name).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            let result = serde_json::from_str(&contents);
            match result {
                Ok(v) => return v,
                Err(_e) => {
                    println!("[{}] failed to open persistance", rank);
                },
            }
        }
        return RaftPersistence {
            current_term: 0,
            voted_for: -1,
            log: Vec::new()
        }
    }

    // * write to file
    fn update(&self, rank: Rank) -> Result<()>{
        let serialized_str = serde_json::to_string(&self)?;
        let serialized_me = serialized_str.as_bytes();
        let file_name = format!("server.{}.persistence", rank);
        let mut file = File::create(&file_name)?;
        file.write_all(serialized_me)?;
        Ok(())
    }
}

pub trait Server {
    fn new(rank: Rank) -> Self;
    fn init_leader_state(&mut self, size: Rank);
    fn write_persistence(&mut self) -> Result<()>;
    fn update_term(&mut self) -> Result<()>;
    fn request_vote(&self) -> rpc::RPCMessage;
    fn empty_append(&self) -> rpc::RPCMessage;
    fn last_log_index(&self) -> usize;
    fn last_log_term(&self) -> u64;
}

#[derive(Debug)]
pub struct RaftServer {
    pub persistence: RaftPersistence,
    // * volatile data for all servers
    pub state: ServerState,
    pub rank: Rank,
    pub commit_index: usize,
    pub last_applied: usize,
    pub vote_count: Rank,
    // * volatile data on leaders
    pub next_index: HashMap<Rank, usize>,
    pub match_index: HashMap<Rank, usize>
}

impl Server for RaftServer {
    fn new(rank: Rank) -> RaftServer {
        RaftServer {
            persistence: RaftPersistence::new(rank),
            state: ServerState::Follower,
            rank: rank,
            commit_index: 0,
            last_applied: 0,
            vote_count: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
        }
    }

    fn init_leader_state(&mut self, size: Rank) {
        let last_log_index = self.persistence.log.len();
        for rank in 0..size {
            self.next_index.insert(rank, last_log_index + 1);
            self.match_index.insert(rank, 0);
        }
    }

    fn write_persistence(&mut self) -> Result<()>{
        self.persistence.update(self.rank)?;
        Ok(())
    }

    fn update_term(&mut self) -> Result<()>{
        self.persistence.current_term = self.persistence.current_term + 1;
        self.write_persistence()
    }

    fn last_log_index(&self) -> usize {
        return self.persistence.log.len();
    }

    fn last_log_term(&self) -> u64 {
        if self.persistence.log.len() != 0 {
            return self.persistence.log[self.persistence.log.len() - 1].term;
        }else{
            return 0;
        }
    }

    fn request_vote(&self) -> rpc::RPCMessage {
        let mut last_term = 0;
        if self.persistence.log.len() != 0 {
            last_term = self.persistence.log[self.persistence.log.len() - 1].term;
        }

        rpc::RPCMessage {
            message: rpc::Request,
            rtype: rpc::RPCType::RequestVote,
            rv_params: Some(rpc::RequestVoteParameters{
                term: self.persistence.current_term,
                candidate_id: self.rank,
                last_log_index: self.persistence.log.len(),
                last_log_term: last_term
            }),
            ae_params: None
        }
    }

    fn empty_append(&self) -> rpc::RPCMessage {
        let mut last_term = 0;
        if self.persistence.log.len() != 0 {
            last_term = self.persistence.log[self.persistence.log.len() - 1].term;
        }
        rpc::RPCMessage {
            message: rpc::Request,
            rtype: rpc::RPCType::AppendEntries,
            rv_params: None,
            ae_params: Some(rpc::AppendEntriesParameters{
                term: self.persistence.current_term,
                leader_id: self.rank,
                prev_log_index: self.persistence.log.len(),
                prev_log_term: last_term,
                entries: vec!(),
                leader_commit: self.commit_index
            })
        }
    }
}
