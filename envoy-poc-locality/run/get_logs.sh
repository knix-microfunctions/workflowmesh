#!/bin/sh
set -ex
if [ -d "$1" ]; then
  echo "$1 dir exists."
  exit 1
fi
rm -rf $1
mkdir $1
cd $1
scp slfu44:~/envoy-poc-locality/run/host1.log .
scp slfu46:~/envoy-poc-locality/run/host2.log .
cat host1.log > combined.log
echo ""  >> combined.log
cat host2.log >> combined.log
cat combined.log | grep TRACE > combined_trace.log
scp slfu45:~/envoy-poc-locality/run/requests.txt .

xargs -L 1 -a ./requests.txt python3 ../process_request_id.py ./combined_trace.log > results_step1.txt
