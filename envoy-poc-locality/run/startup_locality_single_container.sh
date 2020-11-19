#!/bin/sh

HOST_IP=`hostname -i` /frontend &
/service --port 12341 --servicenumber 1 --hostip `hostname -i` --servicetype produce &
/service --port 12342 --servicenumber 2 --hostip `hostname -i` --servicetype process &
/usr/local/bin/envoy -c /envoy-config-locality-single-container.yaml --log-level debug --log-path /envoy.log &
wait $!