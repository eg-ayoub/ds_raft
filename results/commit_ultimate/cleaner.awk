#!/usr/bin/awk -f

{
  if ($2 < 1000) 
    print $0;
}
