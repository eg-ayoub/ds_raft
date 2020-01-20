#!/usr/bin/awk -f

BEGIN {
  lm = -1;
  ls = -1;
  ln = -1;
  rank = 0;
  candidate = 0;
}

/sends/ {
  split($1, a, ":");
  split(a[2], b, ",");

  if (lm == -1) {
    lm = a[1];
    ls = b[1];
    ln = b[2];
    rank = rank + 1;
  } else {
    result = ((a[1] - lm)*60 + b[1]-ls)*1000 + (b[2]-ln)/1000000; 
    if (result > 0) {
      print rank " " result " " 225;   
    }
    lm = a[1];
    ls = b[1];
    ln = b[2];
    rank = rank + 1;
  }
}
