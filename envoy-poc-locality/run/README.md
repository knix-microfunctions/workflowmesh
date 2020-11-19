### How to run the experiments

The experiments (unloaded and loaded) should be run on two different hosts, referred to as `host1` and `host2` in this description. IP addresses and hostnames of both the hosts are required.

#### On `host1` and `host2`

```bash
./build_locality.sh
```

#### On `host1`

Edit `run_locality_distributed_1.sh`:
* Modify the env varaible `REMOTE_ENVOY_HOST=192.168.33.22` in the `docker run` command to point to the IP address of `host2`

```bash
# Launch the first replica of the workflow container
./run_locality_distributed_1.sh > host1.log 2>&1
```

#### On `host2`

Edit `run_locality_distributed_2.sh`: 
* Modify the env variable `REMOTE_ENVOY_HOST=192.168.33.20` in the `docker run` command to point to the IP address of `host1`

```bash
# Launch the second replica of the workflow container
./run_locality_distributed_2.sh > host2.log 2>&1
```

### Run the unloaded experiment

On `host1`, edit `run_unloaded_exp.py`:
* Modify the http urls in the `local_url` and `remote_url` variables to point to the hostnames of `host1` and `host2` respectively

```bash
# Start the unloaded experiment on host1
python3 run_unloaded_exp.py 2001 > requests.txt
```

On `host1`, edit `get_logs.sh`:
* Modify the `scp` command (line 11), to point to the hostname of `host2`

```bash
# from inside the `run` folder
./get_logs.sh unloaded_500kb_to_512mb_16loops

# from inside the `run` folder
docker run -it -v $(pwd):/code -w /code python python3 process_step1_results.py ./unloaded_500kb_to_512mb_16loops/results_step1.txt > ./unloaded_500kb_to_512mb_16loops/results_step2.txt

# refer to `results_in_ppt/results_unloaded_and_loaded_500kb_to_512mb_16loops.xlsx` to see how to plot graphs
```

### Run the loaded experiment

On `host1`, edit `run_loaded_exp.py`:
* Modify the http urls in the `local_url` and `remote_url` variables to point to the hostname of `host1`.

```bash
# Start the loaded experiment on host1
python3 run_loaded_exp.py 8001 > requests.txt
```

On `host1`, edit `get_logs.sh`:
* Modify the `scp` command (line 11), to point to the hostname of `host2`

```bash
# from inside the `run` folder
./get_logs.sh loaded_500kb_to_512mb_16loops

# from inside the `run/loaded_500kb_to_512mb_16loops` folder
cat results_step1.txt | awk '{c=c+1; if (c == 3) {print $0; c=0}}' > results_step1_filtered.txt

# from inside the `run` folder
docker run -it -v $(pwd):/code -w /code python python3 process_step1_results.py ./loaded_500kb_to_512mb_16loops/results_step1_filtered.txt > ./loaded_500kb_to_512mb_16loops/results_step2.txt

# refer to `results_in_ppt/results_unloaded_and_loaded_500kb_to_512mb_16loops.xlsx` to see how to plot graphs
```
