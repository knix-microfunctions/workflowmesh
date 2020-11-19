#!/bin/sh
set -ex

./get_envoy.sh

cd ../service
cargo build --release
cd -
cp ../service/target/release/service .

cd ../frontend
cargo build --release
cd -
cp ../frontend/target/release/frontend .
docker build -t 'envoypoc-locality' --build-arg HTTPS_PROXY=$HTTPS_PROXY --build-arg HTTP_PROXY=$HTTP_PROXY --build-arg http_proxy=$http_proxy --build-arg https_proxy=$https_proxy -f Dockerfile-envoy-poc-locality-rust.dockerfile .
