#!/usr/bin/awk -f 

NR > 1 {
  node = ($1+$6+$11+$16)/4;   
  
  ecart = sqrt(($2^2+$7^2+$12^2+$176^2)/4);

  moy = ($3+$8+$13+$18)/4;

  min = $4;
  if (min > $9)
    min = $9;
  if (min > $14)
    min = $14;
  if (min > $19)
    min = $19;

  max = $5;
  if (max < $10)
    max = $10;
  if (max < $15)
    max = $15;
  if (max < $20)
    max = $20;

  print node " " ecart " " moy " " min " " max;
}
