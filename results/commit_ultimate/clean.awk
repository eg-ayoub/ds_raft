#!/usr/bin/awk -f

/sends/ {
  print $1 " " $2 " " $3 " " $4 " " $5 " " substr(substr($6, 2), 1, length($6)-2);
}

!/sends/ {
  print $0;
}
