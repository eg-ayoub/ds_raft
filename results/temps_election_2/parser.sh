#!/bin/bash

echo '' > means.result

for f in *.log; do
  FILE=${f%.*}
  NODE=`cut -d'-' -f2 <<<"$FILE"`
  
  ./script.awk $f > $FILE.result
  echo $NODE " " `./mean.awk $FILE.result` >> means.result

  gnuplot -e "set title 'Temps entre chaque election pour $NODE noeuds'; \
set xlabel 'Numero election'; \
set ylabel 'Temps election en ms'; \
set yrange [0:1000]; set term png; \
set output '$FILE.png'; plot '$FILE.result';" 

done

gnuplot -e "set title 'Temps Moyen Election par nombre de noeuds'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Temps moyen election en ms'; \
set term png; set output 'means.png'; \
set boxwidth 5; set style fill solid; \
set yrange [0:500]; \
set xrange [0:130]; \
plot 'means.result' using 1:3:xtic(1) with boxes;" 

gnuplot -e "set title 'Nombre Elections par nombre de noeuds'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Nombre Elections'; \
set term png; set output 'elections.png'; \
set boxwidth 5; set style fill solid; \
set xrange [0:130]; \
plot 'means.result' using 1:4:xticlabels(1) with boxes lt rgb '#406090', '' using 1:2 with boxes lt rgb '#40FF00';" 

gnuplot -e "set title 'Nombre Elections par nombre de noeuds'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Nombre Elections'; \
set term png; set output 'elections.png'; \
set boxwidth 5; set style fill solid; \
set xrange [0:130]; \
set yrange[0:200]; \
plot 'means.result' using 4:xticlabels(1) with boxes lt rgb '#406090', '' using 1:2 with boxes lt rgb '#40FF00';" 
