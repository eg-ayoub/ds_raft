#!/bin/bash

for d in ex_*; do
  echo '' > $d/means.result
  echo '' > $d/candidate.result
  for f in $d/*.log; do
    FILE=${f%.*}
    NODE=`cut -d'-' -f2 <<<"$FILE"`
    
    ./script.awk $f > $FILE.result
    echo $NODE " " `./mean.awk $FILE.result` >> $d/means.result
    echo $NODE " " `./candidate.awk $FILE.log` >> $d/candidate.result

    gnuplot -e "set title 'Temps entre chaque election pour $NODE noeuds'; \
set xlabel 'Numero election'; \
set ylabel 'Temps election en ms'; \
set yrange [0:1000]; set term png; \
set output '$FILE.png'; \
plot '$FILE.result' using 1:2, '' using 1:3 with line title 'Ideal';" 

  done

  gnuplot -e "set title 'Temps Moyen Election par nombre de noeuds'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Temps moyen election en ms'; \
set term png; set output '$d/means.png'; \
set boxwidth 5; set style fill solid; \
set yrange [0:500]; \
plot '< sort -n $d/means.result' using 3:xtic(1) with line, \
'' using 4:xtic(1) with line title 'Ideal';" 

  gnuplot -e "set title 'Throughput'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Nombre d élection par secondes'; \
set term png; set output '$d/throughtput.png'; \
set boxwidth 5; set style fill solid; \
set yrange [0:6]; \
plot '< sort -n $d/means.result' using 6:xtic(1) with line title 'Throughput', \
'' using 5:xtic(1) with line title 'Ideal';" 

  gnuplot -e "set title 'Nombre de candidature par rapport au nombre d election'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Instance'; \
set term png; set output '$d/candidate.png'; \
set boxwidth 5; \
set style fill solid; \
set yrange [0:500]; \
plot '< sort -n $d/candidate.result' using 2:xtic(1) with line title 'Elections', \
'' using 3:xtic(1) with line title 'Candidature', \
'' using 6:xtic(1) with line title 'Recandidature';"

  gnuplot -e "set title 'Succès des candidatures'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Taux de succès'; \
set term png; set output '$d/success.png'; \
set boxwidth 5; \
set style fill solid; \
set yrange [0:1.1]; \
set lmargin 10; \
set rmargin 10; \
plot '< sort -n $d/candidate.result' using 4:xtic(1) with line title 'Taux', \
'' using 5:xtic(1) with line title 'Ideal';"
done
