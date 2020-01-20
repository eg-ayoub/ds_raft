#!/bin/bash 

for d in ex_*; do
  for l in $d/log-*; do
    ./parse.awk $l > $l-new
    mv $l-new $l
  done
done
