#!/bin/sh
set -ex
./build_locality.sh
#docker network create --driver bridge --subnet 172.19.0.0/16 --gateway 172.19.0.1 envoy-poc-locality || true
docker run --rm --init --network host --cpus="2.0" --name=wf2 -e "ENVOY_PORT=82" -e "REMOTE_ENVOY_HOST=192.168.33.20" -e "REMOTE_ENVOY_PORT=82" envoypoc-locality /startup_locality_distributed_2.sh
