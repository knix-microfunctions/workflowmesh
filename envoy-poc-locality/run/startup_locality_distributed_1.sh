#!/bin/sh

export HOST_IP=`hostname -i | awk '{print $1}'`
python3 /gen_envoy_config.py /envoy-config-locality-distributed.yaml.j2 /envoy-config-locality-distributed-1.yaml `echo $ENVOY_PORT` `echo $REMOTE_ENVOY_HOST` `echo $REMOTE_ENVOY_PORT` 200
/frontend &
/service --port 12341 --servicenumber 1 --servicetype produce > service1.log &
/service --port 12342 --servicenumber 2 --servicetype process > service2.log &
/usr/local/bin/envoy -c /envoy-config-locality-distributed-1.yaml --log-level info --log-path /envoy.log &
wait $!
