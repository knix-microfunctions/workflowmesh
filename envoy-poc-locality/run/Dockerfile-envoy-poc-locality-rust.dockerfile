FROM rust:1.47-slim-buster
RUN apt-get update
RUN apt-get install -y apt-utils net-tools vim httpie curl wget inetutils-ping
ADD ./envoy /usr/local/bin/
ADD ./startup_locality_single_container.sh /
ADD ./envoy-config-locality-single-container.yaml /
ADD ./gen_envoy_config.py /
ADD ./envoy-config-locality-distributed.yaml.j2 /
ADD ./startup_locality_distributed_1.sh /
ADD ./startup_locality_distributed_2.sh /
ADD ./service /
ADD ./frontend /
CMD ["/startup_locality_single_container.sh"]