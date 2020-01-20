#!/bin/bash

np=(8 16 32 48 64 128 192 256)

for i in {1..5}; do
  module load openmpi/4.0.1_gcc-6.4.0
  for n in ${np[@]}; do
    timeout 1m $(which mpirun) --mca orte_rsh_agent "oarsh" --mca "orte_base_help_aggregate" 0 -machinefile $OAR_NODEFILE -n $n ds_raft > log-$n.log
    mkdir output-$n
    mv server.* output-$n
  done
  mkdir ex_$i
  mv log-* output-* ex_$i/
done
