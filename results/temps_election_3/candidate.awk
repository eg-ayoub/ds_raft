#!/usr/bin/awk -f

BEGIN {
  candidate = 0;
  leader = 0;
  recandidate = 0;
}

/starting/ {
  candidate = candidate + 1;
}

/leader/ {
  leader = leader + 1;
}

/candidate/ {
  recandidate = recandidate + 1;
}

END {
  success = leader / candidate;
  print leader " " candidate " " success " " 1 " " recandidate; 
}
