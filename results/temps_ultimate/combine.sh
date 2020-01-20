#!/bin/bash

paste ex_1/means.result ex_2/means.result ex_3/means.result ex_4/means.result ex_5/means.result | ./combine.awk > means.result
paste ex_1/candidate.result ex_2/candidate.result ex_3/candidate.result ex_4/candidate.result ex_5/candidate.result | ./combine.awk > candidate.result

  gnuplot -e "set title 'Temps Moyen Election par nombre de noeuds'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Temps moyen election en ms'; \
set term png; set output 'means.png'; \
set boxwidth 5; set style fill solid; \
set yrange [0:500]; \
plot '< sort -n means.result' using 3:xtic(1) with line, \
'' using 4:xtic(1) with line title 'Ideal';" 

  gnuplot -e "set title 'Throughput'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Nombre d élection par secondes'; \
set term png; set output 'throughtput.png'; \
set boxwidth 5; set style fill solid; \
set yrange [0:6]; \
plot '< sort -n means.result' using 6:xtic(1) with line title 'Throughput', \
'' using 5:xtic(1) with line title 'Ideal';" 

  gnuplot -e "set title 'Nombre de candidature par rapport au nombre d election'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Instance'; \
set term png; set output 'candidate.png'; \
set style fill solid; \
set yrange [0:500]; \
plot '< sort -n candidate.result' using 2:xtic(1) with line title 'Elections', \
'' using 3:xtic(1) with line title 'Candidature', \
'' using 6:xtic(1) with line title 'Recandidature';"

  gnuplot -e "set title 'Succès des candidatures'; \
set xlabel 'Nombre de noeuds'; \
set ylabel 'Taux de succès'; \
set term png; set output 'success.png'; \
set boxwidth 5; \
set style fill solid; \
set yrange [0:1.1]; \
set lmargin 10; \
set rmargin 10; \
plot '< sort -n candidate.result' using 4:xtic(1) with line title 'Taux', \
'' using 5:xtic(1) with line title 'Ideal';"
