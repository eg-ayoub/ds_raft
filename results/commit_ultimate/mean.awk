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
  avg = total / num;
  print avg;  
}
