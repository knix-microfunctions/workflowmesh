set -ex
cargo build --release
export HOST_IP=`hostname -i | awk '{print $1}'`
export ENVOY_PORT="80"
./target/release/frontend
