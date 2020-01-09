extern crate mpi;
extern crate rand;

mod raft_server;
use std::sync::Arc;
use mpi::point_to_point::*;
use mpi::traits::*;
use mpi::Threading;
use std::thread;
use std::sync::Mutex;
use std::time::Duration;
use std::sync::mpsc::{self, RecvTimeoutError};
use raft_server::server::*;
use raft_server::rpc::*;
use rand::Rng;
use threadpool::ThreadPool;
use mpi::topology::Rank;
use std::collections::VecDeque;




fn main() {

    // * cross-thread communication & MPI
    let (election_sender, election_receiver) = mpsc::channel();
    let out_queue : VecDeque<RPCResponse> = VecDeque::new();
    let out_queue_mut = Arc::new(Mutex::new(out_queue));
    let (universe, _threading) = mpi::initialize_with_threading(Threading::Multiple).unwrap();
    let universe_arc = Arc::new(universe);
    let world = universe_arc.world();
    let size = world.size();
    let rank = world.rank();
    
    if size < 2 {
        panic!("world size must be 2 at least!");
    }

    // * raft server stuff 
    let server = RaftServer::new(rank);
    let server_mut_arc = Arc::new(Mutex::new(server));

    fn get_state(server: Arc<Mutex<RaftServer>>) -> ServerState {
        let server = server.lock().unwrap();
        return server.state;
    }

    // * election heartbeat
    let mut rng = rand::thread_rng();
    let election_timeout: Duration = Duration::from_millis(rng.gen_range(150, 301));
    
    // * MAIN THREAD
    let universe_arc_main = Arc::clone(&universe_arc);
    let server_arc_main = Arc::clone(&server_mut_arc);
    let main_thread = thread::spawn(move || {
        loop {
            match get_state(server_arc_main.clone()) {
                ServerState::Follower => {
                    match election_receiver.recv_timeout(election_timeout) {
                        Err(RecvTimeoutError::Timeout) => {
                            println!("[{}] has timed out. starting new election...", rank);
                            let mut server = server_arc_main.lock().unwrap();
                            server.persistence.current_term = server.persistence.current_term + 1;
                            server.persistence.voted_for = rank;
                            server.vote_count = server.vote_count + 1; 
                            server.state = ServerState::Candidate;
                            let _msg = server.request_vote();
                            let msg = serde_json::to_string(&_msg).unwrap();
                            // * bcast 
                            for r in 0..size {
                                if r != rank {
                                    universe_arc_main.world().process_at_rank(r).send(msg.as_bytes());
                                }
                            }
                            println!("[{}] has broadcasted its requestvote RPC", rank);

                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            panic!("[{}] panicked because it's election signal channel has failed (on receive)", rank);
                        }
                        Ok(_) => {
                            // println!("[{}] has received an RPC", rank);
                        }
                    }
                }
                ServerState::Leader => {
                    let server = server_arc_main.lock().unwrap();
                    thread::sleep(Duration::from_millis(100));
                    let _msg = server.empty_append();
                        let msg = serde_json::to_string(&_msg).unwrap();
                        // * bcast 
                        for r in 0..size {
                            if r != rank {
                                universe_arc_main.world().process_at_rank(r).send(msg.as_bytes());
                            }
                        }
                }
                ServerState::Candidate => {
                    let mut server = server_arc_main.lock().unwrap();
                    if size - server.vote_count < server.vote_count {
                        println!("[{}] becomes leader.", rank);
                        server.state = ServerState::Leader;
                        server.init_leader_state(size);
                        let _msg = server.empty_append();
                        let msg = serde_json::to_string(&_msg).unwrap();
                        // * bcast 
                        for r in 0..size {
                            if r != rank {
                                universe_arc_main.world().process_at_rank(r).send(msg.as_bytes());
                            }
                        }
                    }
                }
            }
        }
    });

    // * INGRESS THREAD gathers incoming messages and issues them 
    let universe_arc_ingress = Arc::clone(&universe_arc);
    let server_arc_ingress = Arc::clone(&server_mut_arc);
    let out_queue_mut_ingress = Arc::clone(&out_queue_mut);
    let ingress_thread = thread::spawn(move || {

        // * define the jobs
        fn issue_requestvote(call: RPCMessage, server: Arc<Mutex<RaftServer>>, out_queue: Arc<Mutex<VecDeque<RPCResponse>>>, from: Rank) {

            let mut server = server.lock().unwrap();

            let mut response = RPCResponse {
                call_type : RPCType::RequestVote,
                response_parameters : RPCResponseParameters {
                    rv_term : server.persistence.current_term,
                    rv_votegranted : true
                },
                to: from
            };

            if call.call_parameters.rv_term < server.persistence.current_term {
                response.response_parameters.rv_votegranted = false;
            } 
            let voted_bool = server.persistence.voted_for == -1 
                || server.persistence.voted_for == call.call_parameters.rv_candidate_id;

            let mut candidate_more_up_to_date = false;
            if call.call_parameters.rv_last_log_term > server.last_log_term() {
                candidate_more_up_to_date = true;
            } else if call.call_parameters.rv_last_log_term == server.last_log_term() {
                candidate_more_up_to_date =  call.call_parameters.rv_last_log_index >= server.last_log_index();
            }
            if !(voted_bool && candidate_more_up_to_date) {
                response.response_parameters.rv_votegranted = false;
            }

            if let Err(_) = server.write_persistence() {
                panic!("[{}] could not write to disk ...");
            }
            println!("[{}] responds {:?}", server.rank, response);
            out_queue.lock().unwrap().push_back(response);
        }

        fn issue_appendentries(_call: RPCMessage, server: Arc<Mutex<RaftServer>>) {
            let _server = server.lock().unwrap();
            // TODO
        }

        // * start the pool of workers
        let n_workers = 4;
        let pool = ThreadPool::new(n_workers);

        loop {
            // * get the message and deserialize it
            let (message, status) = universe_arc_ingress.world().any_process().receive_vec::<u8>();
            let from = status.source_rank();
            let message_string = std::str::from_utf8(&message[..]).unwrap();
            let msg_as_rpc = serde_json::from_str::<RPCMessage>(message_string); 

            // * is the message an RPC?
            if let Err(_) = msg_as_rpc {
            } else {
                let msg_as_rpc = msg_as_rpc.unwrap();
                match msg_as_rpc.call_type {
                    RPCType::RequestVote => {
                        let server_arc_rv = Arc::clone(&server_arc_ingress);
                        let out_queue = out_queue_mut_ingress.clone();
                        pool.execute(move || issue_requestvote(msg_as_rpc, server_arc_rv, out_queue, from));
                    },
                    RPCType::AppendEntries => {
                        let server_arc_ae = Arc::clone(&server_arc_ingress);
                        pool.execute(move || issue_appendentries(msg_as_rpc, server_arc_ae));
                    }
                }
                match election_sender.send(()) {
                    Ok(_v) => {},
                    Err(_e) => panic!("[{}] panicked because it's election signal channel has failed (on send)", rank)
                }
            }

            // * is this message a response? 
            let msg_as_resp = serde_json::from_str::<RPCResponse>(message_string);
            if let Err(_) = msg_as_resp {
                // ! 
            } else {
                let msg_as_resp = msg_as_resp.unwrap();
                match msg_as_resp.call_type {
                    RPCType::RequestVote => {
                        if msg_as_resp.response_parameters.rv_votegranted {
                            let mut server = server_arc_ingress.lock().unwrap();
                            server.vote_count = server.vote_count + 1;
                        }
                    },
                    RPCType::AppendEntries => {
                    }
                }
            }
            
        }
    });

    // * OUTGRESS THREAD sends RPC responses back to their senders
    let universe_arc_out = Arc::clone(&universe_arc);
    let out_queue_mut_out = Arc::clone(&out_queue_mut);
    let outgress_thread = thread::spawn(move || {
        loop {
            let msg = out_queue_mut_out.lock().unwrap().pop_front();
            match msg {
                Some(m) => {
                    let msg_str = serde_json::to_string(&m).unwrap();
                    universe_arc_out.world().process_at_rank(m.to).send(msg_str.as_bytes());
                },
                None => {}
            }
        }
    });


    outgress_thread.join().unwrap();
    ingress_thread.join().unwrap();
    main_thread.join().unwrap();

    
}

