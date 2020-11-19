### How to run the experiments

The experiments (unloaded and loaded) should be run on two different hosts, referred to as `host1` and `host2` in this description. IP addresses and hostnames of both the hosts are required.

#### On `host1` and `host2`
```bash
./build_locality.sh
```

#### On `host1`

Edit `run_locality_distributed_1.sh`. Modify the `REMOTE_ENVOY_HOST=192.168.33.22` in the `docker run` command to point to the IP address of `host2`

```bash

./run_locality_distributed_1.sh > host1.log 2>&1
```

#### On `host2`

Edit `run_locality_distributed_2.sh`. Modify the `REMOTE_ENVOY_HOST=192.168.33.20` in the `docker run` command to point to the IP address of `host1`

```bash
./run_locality_distributed_2.sh > host2.log 2>&1
```

### Run the unloaded experiment

On `host1`, edit `run_unloaded_exp.py`. Modify the http urls in the `local_url` and `remote_url` variables to point to the hostnames of `host1` and `host2`

```bash
python3 run_unloaded_exp.py 2001 > requests.txt
```

Edit `get_logs.sh`. 
./get_logs.sh


python3 run_loaded_exp.py 8001 > requests.txt


    // for loaded
    cat results_step1.txt | awk '{c=c+1; if (c == 3) {print $0; c=0}}' > results_step1_filtered.txt

docker run -it -v $(pwd):/code -w /code python bash

python3 ./process_step1_results.py ./25_50_100_200_400mb/results_step1.txt > ./25_50_100_200_400mb/results_step2.txt
    
    // for loaded
    python3 ./process_step1_results.py loaded_25_50_100_more_compute/results_step1_filtered.txt > loaded_25_50_100_more_compute/results_step2.txt
