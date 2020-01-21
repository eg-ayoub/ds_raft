#!/usr/bin/awk -f

{
  node = $1;
  num = $3;
  ecart = $5;
  moy = $4;

  error = 1.96 * (ecart / sqrt(num));
  min = moy - error;
  max = moy + error;

  print node " " error " " min " " max;
}
