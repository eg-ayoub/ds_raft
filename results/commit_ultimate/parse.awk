#!/usr/bin/awk -f

BEGIN {
  rank = 0;
}

/sends/ {
  time[$6] = $1
}

/leader commited/ {
  split($1, a, ":");
  split(a[2], b, ".")
  lm = a[1];
  ls = b[1];
  ln = b[2];

  split(time[$6], c, ":");
  split(c[2], d, ".");

  rank = rank + 1;

  print rank " " ((lm - c[1])*60 + ls-d[1])*1000 + (ln-d[2])/1000000; 
}
