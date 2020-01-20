BEGIN {
  lm = 0;
  ls = 0;
  ln = 0;
  rank = 0;
}

/Send/ {
  split($1, a, ":");
  split(a[2], b, ".")
  lm = a[1];
  ls = b[1];
  ln = b[2];
  rank = rank + 1;
}

/Commit/ {
  print rank " " ((a[1] - lm)*60 + b[1]-ls)*1000 + (b[2]-ln)/1000000; 
}
