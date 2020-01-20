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
  max  = (5*60000)/mean;
  per  = num/max;
  print num " " mean " " max " " per;
}
