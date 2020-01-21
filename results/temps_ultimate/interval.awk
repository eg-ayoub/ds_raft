#!/usr/bin/awk -f

{
  node = $1;
  num = $2;
  ecart = $8;
  moy = $3;

  error = 1.96 * (ecart / sqrt(num));
  min = moy - error;
  max = moy + error;

  print node " " error " " min " " max;
}
