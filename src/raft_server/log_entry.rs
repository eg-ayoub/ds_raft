use serde::{Serialize, Deserialize};
use std::cmp::min;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum RaftOperation {
    Incr,
    Decr,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct LogEntry {
    pub operation: RaftOperation,
    pub term: u64,
}

pub trait RaftList {
    fn new() -> Self;
    fn len(&self) -> usize;
    fn get(&self, raft_index: usize) -> LogEntry;
    fn find(&self, raft_index: usize, term: u64) -> Result<bool, &'static str>;
    fn put(&mut self, entry: LogEntry) -> usize;
    fn last_term(&self) -> u64;
    fn find_conflicts(&self, prev_index: usize, compare_to: &Vec<LogEntry>) -> Option<usize>;
    fn truncate(&mut self, starting: usize);
    fn append_rest(&mut self, starting: usize, prev: usize, compare_to: &Vec<LogEntry>);
    fn append_all(&mut self, prev: usize, compare_to: &Vec<LogEntry>);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    list: Vec<LogEntry>,
}

impl RaftList for Log{
    fn new() -> Log {
        Log {
            list: Vec::new()
        }
    }

    fn len(&self) -> usize {
        self.list.len()
    }

    fn get(&self, raft_index: usize) -> LogEntry {
        self.list[raft_index-1]
    }

    fn find(&self, raft_index: usize, term: u64) -> Result<bool, &'static str> {
        if raft_index > self.list.len() {
            Err("index out of bounds")
        } else if raft_index == 0 {
            if term != 0 {
                Err("entry at index 0")
            } else {
                Ok(true)
            }
        } else {
            Ok(self.list[raft_index-1].term == term)
        }
    }

    fn put(&mut self, entry: LogEntry) -> usize {
        self.list.push(entry);
        self.list.len()
    }

    fn last_term(&self) -> u64 {
        if self.list.len() != 0 {
            return self.list[self.list.len() - 1].term;
        }else{
            return 0;
        }
    }

    fn find_conflicts(&self, prev_index: usize, compare_to: &Vec<LogEntry>) -> Option<usize> {
        for index in prev_index..min(prev_index + compare_to.len(), self.list.len()) {
            if compare_to[index - prev_index].term != self.list[index].term {
                return Some(index);
            }
        } 
        return None;
    }


    fn truncate(&mut self, starting: usize) {
        self.list.truncate(starting);
    }

    fn append_rest(&mut self, starting: usize, prev: usize, compare_to: &Vec<LogEntry>) {
        for index in starting-prev..compare_to.len() {
            info!("follower appended <{}>", prev + index + 1);
            self.list.push(compare_to[index]);
        }
    }

    fn append_all(&mut self, prev: usize, compare_to: &Vec<LogEntry>) {
        for index in self.len()-prev..compare_to.len() {
            info!("follower appended <{}>", prev + index + 1);
            self.list.push(compare_to[index]);
        }
    }
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        for (count, v) in self.list.iter().enumerate() {
            if count != 0 {
                write!(f, ",")?;
            }
            write!(f, "({}|{}|{:?})", count + 1, v.term, v.operation)?;
        }
        write!(f, "]")
    }
}