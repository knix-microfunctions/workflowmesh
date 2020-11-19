import requests
from copy import deepcopy
import time
import sys
from multiprocessing import Pool

start_req_id = int(sys.argv[1])
assert(start_req_id > 0)

#filesizes = [16*1024*1024, 32*1024*1024]

filesizes = [500*1024, 1*1024*1024, 2*1024*1024, 4*1024*1024, 8*1024*1024, 16 *
             1024*1024, 32*1024*1024, 64*1024*1024, 128*1024*1024, 256*1024*1024, 512*1024*1024]

# (islocal, isfileref) pairs
locality_config = [((True, True), (True, True)), ((True, True), (True, False)), ((
    True, True), (False, True)), ((True, True), (False, False))]
num_experiments_per_config = 5
delay_between_requests = 0.2

base_request_object = {"request_id": "", "pass_data": False, "data": "", "filename": "", "size": 167772160, "loops": 16, "terminal_service_number": 2,
                       "source_service_number": 0, "source_service_ip": "", "source_service_port": 0, "frontend_ip": "", "service_ips": ""}

local_url = "http://host1:82/service/0/start"
remote_url = "http://host1:82/service/0/start"


def make_request(request_id, filesize, islocal, isfileref):
    request_object = deepcopy(base_request_object)

    request_object["request_id"] = request_id
    request_object["size"] = filesize
    request_object["pass_data"] = False  # that means a fileref
    if isfileref == False:
        request_object["pass_data"] = True  # that means a raw data pass

    x_force_local_header_value = '[1,2]'  # keeps it local
    if islocal == False:
        x_force_local_header_value = '[1]'   # forces a remote jump

    headers = {'x-force-local': x_force_local_header_value}
    url = local_url
    if islocal == False:
        url = remote_url

    # print(url)
    # print(str(request_object))
    # print(str(headers))

    t1 = time.time()
    res = requests.post(url, json=request_object, headers=headers)
    t2 = time.time()
    status = res.status_code
    time_diff = t2-t1
    # print(status)
    # print(str(res.json()))

    islocalnum = 0
    if islocal:
        islocalnum = 1
    isfilerefnum = 0
    if isfileref:
        isfilerefnum = 1
    time_diff_str = "%.3f" % time_diff
    req_str = f"{request_id} {filesize} {islocalnum} {isfilerefnum} {status} {time_diff_str}"
    # print(req_str)
    return req_str


def call_make_request(args):
    request_id = args[0]
    datasize = args[1]
    islocal = args[2]
    isfileref = args[3]
    sleep_time = args[4]
    if sleep_time > 0.0:
        time.sleep(sleep_time)
    req_str = make_request(request_id, datasize, islocal, isfileref)
    return req_str


req_number = start_req_id - 1
with Pool(3) as p:
    for datasize in filesizes:
        for config in locality_config:
            for i in range(num_experiments_per_config):
                (islocal_base, isfileref_base) = config[0]
                (islocal_main, isfileref_main) = config[1]
                base_call_args = (str(req_number+1), datasize,
                                  islocal_base, isfileref_base, 0.0)
                base_call_args_2 = (str(req_number+2), datasize,
                                    islocal_base, isfileref_base, 0.0)
                main_call_args = (str(req_number+3), datasize,
                                  islocal_main, isfileref_main, delay_between_requests)

                call_outputs = p.map(call_make_request, [
                                     base_call_args, base_call_args_2, main_call_args])
                print(call_outputs[0])
                print(call_outputs[1])
                print(call_outputs[2])
                sys.stdout.flush()
                req_number = req_number + 3
                time.sleep(5)
