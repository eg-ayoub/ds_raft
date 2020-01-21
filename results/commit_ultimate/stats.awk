#!/usr/bin/awk -f 

BEGIN {
  min = -1;
  max = 0;
  tot = 0;
  num = 0;
  moy = 0;
}

{
  num = num + 1;
  value[num] = $2;

  if (min == -1) {
    min = $2;
  }
  
  if ($2 < min && $2 > 0) {
    min = $2;  
  }

  if ($2 > max) {
    max = $2;
  }

  tot = tot + $2;
}

END {
  moy = tot / num;
  ecart = 0;
  for (i = 1; i <= num; i++) {
    ecart += (value[i] - moy)^2;
  }
  ecart = sqrt(ecart / num);
  print ecart " " moy " " min " " max;
}
