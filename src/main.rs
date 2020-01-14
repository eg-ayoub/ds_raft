extern crate mpi;
extern crate rand;

#[macro_use]
extern crate log;

mod raft_server;
use std::sync::Arc;
use mpi::point_to_point::*;
use mpi::traits::*;
use mpi::Threading;
use std::thread;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::sync::mpsc::{self, RecvTimeoutError};
use raft_server::server::*;
use raft_server::rpc::*;
use rand::Rng;
use threadpool::ThreadPool;
use mpi::topology::Rank;
use std::collections::VecDeque;
use log::{info, warn};



fn main() {
    // * logging
    env_logger::init();

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
        warn!("world size must be 2 at least!");
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
    let election_timeout: Duration = Duration::from_millis(rng.gen_range(1500, 3001));
    let mut start_of_election = Instant::now();
    
    // * MAIN THREAD
    let universe_arc_main = Arc::clone(&universe_arc);
    let server_arc_main = Arc::clone(&server_mut_arc);
    let main_thread = thread::spawn(move || {
        loop {
            match get_state(server_arc_main.clone()) {
                ServerState::Follower => {
                    match election_receiver.recv_timeout(election_timeout) {
                        Err(RecvTimeoutError::Timeout) => {
                            info!("[{}] has timed out. starting new election...", rank);
                            let mut server = server_arc_main.lock().unwrap();
                            server.persistence.current_term = server.persistence.current_term + 1;
                            server.persistence.voted_for = rank;
                            server.vote_count = 1; 
                            server.state = ServerState::Candidate;
                            let _msg = server.request_vote();
                            let msg = serde_json::to_string(&_msg).unwrap();
                            // * bcast 
                            for r in 0..size {
                                if r != rank {
                                    universe_arc_main.world().process_at_rank(r).send(msg.as_bytes());
                                }
                            }
                            start_of_election = Instant::now();
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            warn!("[{}] panicked because it's election signal channel has failed (on receive)", rank);
                        }
                        Ok(_) => {
                            //
                        }
                    }
                },
                ServerState::Leader => {
                    let server = server_arc_main.lock().unwrap();
                    let now = Instant::now();
                    while now.elapsed().as_millis() <= 1000 {
                        thread::yield_now();
                    }
                    let _msg = server.empty_append();
                    let msg = serde_json::to_string(&_msg).unwrap();
                    // * bcast 
                    for r in 0..size {
                        if r != rank {
                            universe_arc_main.world().process_at_rank(r).send(msg.as_bytes());
                        }
                    }
                },
                ServerState::Candidate => {
                    let mut server = server_arc_main.lock().unwrap();
                    if size - server.vote_count < server.vote_count {
                        info!("[{}] becomes leader.", rank);
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
                    }else if start_of_election.elapsed().as_millis() >= 3000 {
                            info!("[{}] candidate has timed out. starting new election...", rank);
                            let mut server = server_arc_main.lock().unwrap();
                            server.persistence.current_term = server.persistence.current_term + 1;
                            server.persistence.voted_for = rank;
                            server.vote_count = 1; 
                            server.state = ServerState::Candidate;
                            let _msg = server.request_vote();
                            let msg = serde_json::to_string(&_msg).unwrap();
                            // * bcast 
                            for r in 0..size {
                                if r != rank {
                                    universe_arc_main.world().process_at_rank(r).send(msg.as_bytes());
                                }
                            }
                            start_of_election = Instant::now();
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
        fn issue_requestvote(params: RequestVoteParameters, server: Arc<Mutex<RaftServer>>, out_queue: Arc<Mutex<VecDeque<RPCResponse>>>, from: Rank) {
            let mut server = server.lock().unwrap();

            let mut response = RPCResponse {
                response: Response,
                rtype : RPCType::RequestVote,
                params : RPCResponseParameters {
                    term : server.persistence.current_term,
                    success : true
                },
                to: from
            };

            if params.term < server.persistence.current_term {
                response.params.success = false;
            }
            
            if params.term > server.persistence.current_term {
                server.persistence.voted_for = -1;
            }

            let voted_bool = server.persistence.voted_for == -1 
                || server.persistence.voted_for == params.candidate_id;

            let mut candidate_more_up_to_date = false;
            if params.last_log_term > server.last_log_term() {
                candidate_more_up_to_date = true;
            } else if params.last_log_term == server.last_log_term() {
                candidate_more_up_to_date =  params.last_log_index >= server.last_log_index();
            }
            if !(voted_bool && candidate_more_up_to_date) {
                response.params.success = false;
            }

            // * Leader discovers new term
            if params.term > server.persistence.current_term && server.state == ServerState::Leader {
                server.state = ServerState::Follower;
            }

            // * update term ? 
            if params.term > server.persistence.current_term {
                server.persistence.current_term = params.term;
            }

            // * update voted for
            if response.params.success {
                server.persistence.voted_for = params.candidate_id;
                response.params.term = server.persistence.current_term;
            }

            if let Err(_) = server.write_persistence() {
                warn!("[{}] could not write to disk ...", server.rank);
            }

            debug!("[{}] responds {} to <{}-RV>", server.rank, response.params.success, response.to);
            out_queue.lock().unwrap().push_back(response);
        }

        fn issue_appendentries(params: AppendEntriesParameters, server: Arc<Mutex<RaftServer>>, out_queue: Arc<Mutex<VecDeque<RPCResponse>>>, from: Rank) {
            let mut server = server.lock().unwrap();
            // TODO
            let mut response = RPCResponse {
                response : Response,
                rtype : RPCType::RequestVote,
                params : RPCResponseParameters {
                    term : server.persistence.current_term,
                    success : true
                },
                to: from
            };

            if params.term < server.persistence.current_term {
                response.params.success = false;
            }

            // if params.term > server.persistence.current_term && server.state != ServerState::Leader {
            //     server.state = ServerState::Follower;
            //     info!("[{}] goes back to being a follower", server.rank);
            // }

            if let Err(_) = server.write_persistence() {
                warn!("[{}] could not write to disk ...", server.rank);
            }

            // info!("[{}] responds {} to <{}-RV>", server.rank, response.params.success, response.to);
            out_queue.lock().unwrap().push_back(response);
        }

        // * start the pool of workers
        let n_workers = 4;
        let pool = ThreadPool::new(n_workers);

        loop {
            // * get the message and deserialize it
            let (message, status) = universe_arc_ingress.world().any_process().receive_vec::<u8>();
            let from = status.source_rank();
            let message_string = std::str::from_utf8(&message[..]).unwrap();
            // info!("[{}] receives {}", rank, message_string);

            // * is the message an RPC?
            let msg_as_rpc = serde_json::from_str::<RPCMessage>(message_string); 
            if let Err(_) = msg_as_rpc {
            } else {
                let msg_as_rpc = msg_as_rpc.unwrap();
                match msg_as_rpc.rtype {
                    RPCType::RequestVote => {
                        let server_arc_rv = Arc::clone(&server_arc_ingress);
                        let out_queue = out_queue_mut_ingress.clone();
                        pool.execute(move || {
                            match msg_as_rpc.rv_params {
                                Some(params) => {
                                    issue_requestvote(params, server_arc_rv, out_queue, from);
                                },
                                None => {}
                            }
                        });
                    },
                    RPCType::AppendEntries => {
                        let server_arc_ae = Arc::clone(&server_arc_ingress);
                        let out_queue = out_queue_mut_ingress.clone();
                        pool.execute(move || {
                            match msg_as_rpc.ae_params {
                                Some(params) => {
                                    issue_appendentries(params, server_arc_ae, out_queue, from);
                                },
                                None => {}
                            }
                        });
                    }
                }
                match election_sender.send(()) {
                    Ok(_v) => {},
                    Err(_e) => warn!("[{}] panicked because it's election signal channel has failed (on send)", rank)
                }
                continue;
            }


            // * is this message a response? 
            let msg_as_resp = serde_json::from_str::<RPCResponse>(message_string);
            if let Err(_) = msg_as_resp {
            } else {
                let msg_as_resp = msg_as_resp.unwrap();
                match msg_as_resp.rtype {
                    RPCType::RequestVote => {
                        let mut server = server_arc_ingress.lock().unwrap();
                        if msg_as_resp.params.success {
                            server.vote_count = server.vote_count + 1;
                        }else if msg_as_resp.params.term > server.persistence.current_term {
                            info!("[{}] candidate goes back to being a follower", server.rank);
                            server.persistence.current_term = msg_as_resp.params.term;
                            server.state = ServerState::Follower;
                            assert_eq!(msg_as_resp.params.term, server.persistence.current_term);
                        }
                    },
                    RPCType::AppendEntries => {
                    }
                }
                continue;
            }
            
            // warn!("server has received unknown message : {}", message_string);
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

