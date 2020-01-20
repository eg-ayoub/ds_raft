use std::collections::HashMap;
use mpi::topology::Rank;
use std::fs::File;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};
use std::io::Result;
use std::cmp::min;

pub use crate::raft_server::log_entry;
pub use crate::raft_server::rpc;

use log_entry::*;

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
    pub log: Log
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
            log: Log::new()
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
    fn custom_append(&self, to: Rank) -> rpc::RPCMessage;
    fn last_log_index(&self) -> usize;
    fn last_log_term(&self) -> u64;
    fn put_in_log(&mut self, entry: LogEntry);
    fn decr_next_index(&mut self, of: Rank);
    fn set_next_index(&mut self, of: Rank, index: usize);
    fn set_match_index(&mut self, of: Rank, index: usize);
    fn update_commit_index(&mut self, size: Rank);
    fn apply_commited(&mut self);
    fn dummy_incr(&mut self);
    fn dummy_decr(&mut self);
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
    pub val: i32,
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
            val: 0,
            next_index: HashMap::new(), // ! should never reach 0
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
        self.persistence.log.len()
    }

    fn last_log_term(&self) -> u64 {
        self.persistence.log.last_term()
    }

    fn request_vote(&self) -> rpc::RPCMessage {
        rpc::RPCMessage {
            message: rpc::Request,
            rtype: rpc::RPCType::RequestVote,
            rv_params: Some(rpc::RequestVoteParameters{
                term: self.persistence.current_term,
                candidate_id: self.rank,
                last_log_index: self.last_log_index(),
                last_log_term: self.last_log_term()
            }),
            ae_params: None
        }
    }

    fn empty_append(&self) -> rpc::RPCMessage {
        rpc::RPCMessage {
            message: rpc::Request,
            rtype: rpc::RPCType::AppendEntries,
            rv_params: None,
            ae_params: Some(rpc::AppendEntriesParameters{
                term: self.persistence.current_term,
                leader_id: self.rank,
                prev_log_index: self.last_log_index(),
                prev_log_term: self.last_log_term(),
                entries: vec!(),
                leader_commit: self.commit_index
            })
        }
    }

    fn custom_append(&self, to: Rank) -> rpc::RPCMessage {
        if self.last_log_index() >= self.next_index[&to] {

            let mut term = 0;
            if self.next_index[&to] > 1 {
                term = self.persistence.log.get(self.next_index[&to] - 1).term;
            }  

            let mut entries = Vec::<LogEntry>::new();
            for i in self.next_index[&to]..self.last_log_index()+1 {
                entries.push(self.persistence.log.get(i));
            }
            return rpc::RPCMessage {
                message: rpc::Request,
                rtype: rpc::RPCType::AppendEntries,
                rv_params: None,
                ae_params: Some(rpc::AppendEntriesParameters{
                    term: self.persistence.current_term,
                    leader_id: self.rank,
                    prev_log_index: self.next_index[&to] - 1,
                    prev_log_term: term,
                    entries: entries,
                    leader_commit: self.commit_index
                })
            };
        }
        return self.empty_append();
    }

    fn put_in_log(&mut self, entry: LogEntry) {
        self.persistence.log.put(entry);
    }

    fn dummy_incr(&mut self) {
        let entry = LogEntry {
            term: self.persistence.current_term,
            operation: RaftOperation::Incr 
        };
        self.put_in_log(entry);
        info!("leader sends <{}>", self.last_log_index());
    }

    fn dummy_decr(&mut self) {
        let entry = LogEntry {
            term: self.persistence.current_term,
            operation: RaftOperation::Decr 
        };
        self.put_in_log(entry);
    }

    fn decr_next_index(&mut self, of: Rank) {
        let new_index = min(self.next_index[&of] - 1, 1);
        self.next_index.remove(&of);
        self.next_index.insert(of, new_index);
    }

    fn set_next_index(&mut self, of: Rank, index: usize) {
        self.next_index.remove(&of);
        self.next_index.insert(of, index);
    }

    fn set_match_index(&mut self, of: Rank, index: usize) {
        self.match_index.remove(&of);
        self.match_index.insert(of, index);
    }

    fn update_commit_index(&mut self, size: Rank) {
        for n in self.commit_index+1..self.persistence.log.len() {
            if self.persistence.log.get(n).term == self.persistence.current_term {
                let mut count = 0;
                for r in 0..size {
                    if r != self.rank && self.match_index[&r] >= n{
                        count = count + 1;
                    }
                }
                if count > size - count {
                    self.commit_index = n;
                    info!("leader commited {}", n);
                }
            }
        }
    }

    fn apply_commited(&mut self) {
        if self.last_applied < self.commit_index {
            self.last_applied = self.last_applied + 1;
            let entry = self.persistence.log.get(self.last_applied);
            match entry.operation {
                RaftOperation::Incr => {
                    self.val = self.val + 1;
                },
                RaftOperation::Decr => {
                    self.val = self.val - 1;
                }
            }
        }
    }


}
