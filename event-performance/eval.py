import numpy as np
import scipy.stats as st
from statistics import mean, median
import os
import sys

if sys.argv[1] in ["seq","kseq","chain","kchain"]:
  NAME = sys.argv[1]+"test"
else:
  NAME="kseqtest"
  #NAME="seqtest"
files = set()
for fn in os.listdir("."):#["chaintest-8388608.csv"]:#["chaintest-results.csv","seqtest-results.csv"]:
  if not fn.startswith(NAME) or not fn.endswith(".csv") or fn == NAME+"-totals.csv" or fn == NAME+"-results.csv":
    continue
  files.add(fn)
out = open(NAME+"-totals.csv","w")
out.write(";".join(["Size","Min","Max","Mean","Median","Conf95start","Conf95end"])+"\n")
for fn in sorted(files):
  #print(fn)
  transfer = {}
  with open(fn) as f:
    for l in f:
      size,ts = l.split(',',1)
      t = ts.split(',')
      times = transfer.setdefault(size,[[],[],[],[],[]])
      times[0].append( (int(t[1])-int(t[0])) / 1000000.0 )
      times[1].append( (int(t[3])-int(t[2])) / 1000000.0 )
      times[2].append( (int(t[5])-int(t[4])) / 1000000.0 )
      times[3].append( (int(t[7])-int(t[6])) / 1000000.0 )
      times[4].append( (int(t[9])-int(t[8])) / 1000000.0 )

  for s,times in transfer.items():
    print("Size %9s:" % s)
    print("STEP         MIN      MAX     MEAN   MEDIAN        ConfInt .95")
    t = [item for sublist in times for item in sublist]
    a = np.array(t)
    ci95 = st.t.interval(0.95, len(a)-1, loc=np.mean(a), scale=st.sem(a))
    print("Total:  %8.3f %8.3f %8.3f %8.3f %8.3f-%8.3f" % (min(t),max(t),mean(t),median(t),ci95[0],ci95[1]))
    out.write(";".join([str(x) for x in [s,min(t),max(t),mean(t),median(t),ci95[0],ci95[1]]])+"\n")
    i=0
    for ts in times:
      a = np.array(ts)
      ci95 = st.t.interval(0.95, len(a)-1, loc=np.mean(a), scale=st.sem(a))
      print("  %1d->%1d: %8.3f %8.3f %8.3f %8.3f %8.3f-%8.3f" % (i, i+1, min(ts),max(ts),mean(ts),median(ts),ci95[0],ci95[1]))
      i+=1
out.close()
