#!/usr/bin/awk -f
NR > 1 {
  print ($1+$4+$7+$10)/4 " " ($2+$5+$8+$11)/4 " " ($3+$6+$9+$12)/4;   
}
