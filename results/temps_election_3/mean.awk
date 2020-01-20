#!/usr/bin/awk -f 

BEGIN {
  total = 0;
  num = 0;
}

{
  total = total + $2;
  num = num + 1;
}

END {
  mean = total/num;
  avg = 225;
  throughput = (60000 / num) / 60;
  print num " " mean " " avg " " 1000/avg " " throughput;
}
