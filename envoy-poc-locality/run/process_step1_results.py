import statistics
import sys

results_step1_file = sys.argv[1]
num_experiments_per_config = 5
# locality_config = [(True, True), (True, False), (False, True), (False, False)]
locality_config = [(1, 1), (1, 0), (0, 1), (0, 0)]
filesizes = [500*1024, 1*1024*1024, 2*1024*1024, 4*1024*1024, 8*1024*1024, 16 *
             1024*1024, 32*1024*1024, 64*1024*1024, 128*1024*1024, 256*1024*1024, 512*1024*1024]

loglines = []
with open(results_step1_file, "r") as f:
    loglines = f.readlines()

c = -1

for datasize in filesizes:
    datasize_result = [datasize]
    req_times_results = []
    req_times_results.append("roundtrip")
    proc_times_results = []
    proc_times_results.append("processing")
    call_times_results = []
    call_times_results.append("call_f1_f2")
    fetch_times_results = []
    fetch_times_results.append("fetch_file")

    for (islocal, isfileref) in locality_config:
        islocalstr = 'local'
        if islocal == 0:
            islocalstr = 'remote'
        isfilerefstr = 'fileref'
        if isfileref == 0:
            isfilerefstr = 'datacall'
        req_times = []
        req_times_results.append(islocalstr)
        req_times_results.append(isfilerefstr)

        proc_times = []
        proc_times_results.append(islocalstr)
        proc_times_results.append(isfilerefstr)

        call_times = []
        call_times_results.append(islocalstr)
        call_times_results.append(isfilerefstr)

        fetch_times = []
        fetch_times_results.append(islocalstr)
        fetch_times_results.append(isfilerefstr)
        for i in range(num_experiments_per_config):
            c = c+1
            one_result = loglines[c]
            one_result = one_result.strip().split(",")

            assert(datasize == int(one_result[1]))
            assert(islocal == int(one_result[2]))
            assert(isfileref == int(one_result[3]))

            assert(float(one_result[4]) >= 0.0)
            req_times.append(float(one_result[4]))

            assert(float(one_result[5]) >= 0.0)
            proc_times.append(float(one_result[5]))

            assert(float(one_result[6]) >= 0.0)
            call_times.append(float(one_result[6]))

            assert(float(one_result[7]) >= 0.0)
            fetch_times.append(float(one_result[7]))

        req_times_results.append(round(statistics.mean(req_times), 4))
        # req_times_results.append(statistics.median(req_times))
        for q in statistics.quantiles(req_times, n=4):
            req_times_results.append(round(q, 4))

        proc_times_results.append(round(statistics.mean(proc_times), 4))
        # proc_times_results.append(statistics.median(proc_times))
        for q in statistics.quantiles(proc_times, n=4):
            proc_times_results.append(round(q, 4))

        call_times_results.append(round(statistics.mean(call_times), 4))
        # call_times_results.append(statistics.median(call_times))
        for q in statistics.quantiles(call_times, n=4):
            call_times_results.append(round(q, 4))

        fetch_times_results.append(round(statistics.mean(fetch_times), 4))
        # fetch_times_results.append(statistics.median(fetch_times))
        for q in statistics.quantiles(fetch_times, n=4):
            fetch_times_results.append(round(q, 4))

    for v in req_times_results:
        datasize_result.append(v)
    for v in proc_times_results:
        datasize_result.append(v)
    for v in call_times_results:
        datasize_result.append(v)
    for v in fetch_times_results:
        datasize_result.append(v)

    datasize_result_str = ""
    for v in datasize_result:
        datasize_result_str = datasize_result_str + f"{v};"
    datasize_result_str = datasize_result_str[:-1]
    print(datasize_result_str)
