#!/bin/bash

for d in ex_*; do
  echo '' > $d/means.result
  echo '' > $d/stats.result
  for l in $d/*.log;do
    FILE=${l%.*}
    NODE=`cut -d'-' -f2 <<<"$FILE"`

    ./parse.awk $l > $FILE.result
    ./cleaner.awk $FILE.result > $FILE.result.new
    mv $FILE.result.new $FILE.result
    echo $NODE " " `./mean.awk $FILE.result` >> $d/means.result
    echo $NODE " " `./stats.awk $FILE.result` >> $d/stats.result

    gnuplot -e "set title 'Temps du commit du leader'; \
set xlabel 'Numero du commit'; \
set ylabel 'Temps du commit en ms'; \
set yrange [200:300]; set term png; \
set output '$FILE.png'; \
plot '$FILE.result' using 1:2;" 

  done

  gnuplot -e "set title 'Temps moyen du commit du leader'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Temps moyen commit en ms'; \
set term png; set output '$d/means.png'; \
set boxwidth 5; set style fill solid; \
set yrange [200:400]; \
plot '< sort -n $d/means.result' using 2:xtic(1) with line;"
done

paste ex_1/means.result ex_2/means.result ex_4/means.result ex_5/means.result | ./combine.awk > means.result
paste ex_1/stats.result ex_2/stats.result ex_4/stats.result ex_5/stats.result | ./combine_stats.awk > stats.result

paste means.result stats.result | ./interval.awk > interval.result

  gnuplot -e "set title 'Temps Moyen du commit du leader'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Temps moyen du commit en ms'; \
set term png; set output 'means.png'; \
set boxwidth 5; set style fill solid; \
set yrange [200:260]; \
plot '< sort -n means.result' using 2:xtic(1) with line title 'Temps'; "
