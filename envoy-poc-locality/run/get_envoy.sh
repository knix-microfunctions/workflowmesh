#!/bin/sh
docker run -it --rm -u $(id -u):$(id -g) -v $(pwd):/code -w /code envoyproxy/envoy bash -c 'cp /usr/local/bin/envoy .'
