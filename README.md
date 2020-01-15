# ds_raft
raft implementation in rust

# build :
```cargo build```

# launch :
```mpirun -n X target/debug/ds_raft```
with X as the number of nodes

# Experience proposition :
 - simulate a lag in network and see how much it takes to have a new leader
 - Simulate a leader kill and see how much time it takes to have a new leader
 - Simulate a new user poping in and see how much time it takes for the convergence
