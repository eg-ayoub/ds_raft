#!/usr/bin/awk -f 
{
  node = ($1+$6+$11+$16+$21)/5;   
  
  ecart = sqrt(($2^2+$7^2+$12^2+$176^2+$22^2)/5);

  moy = ($3+$8+$13+$18+$23)/5;

  min = $4;
  if (min > $9)
    min = $9;
  if (min > $14)
    min = $14;
  if (min > $19)
    min = $19;
  if (min > $24)
    min = $24;

  max = $5;
  if (max < $10)
    max = $10;
  if (max < $15)
    max = $15;
  if (max < $20)
    max = $20;
  if (max < $25)
    max = $25;

  print node " " ecart " " moy " " min " " max;
}
