import requests
from copy import deepcopy
import time
import sys

start_req_id = int(sys.argv[1])
assert(start_req_id > 0)

#filesizes = [16*1024*1024, 32*1024*1024]

filesizes = [500*1024, 1*1024*1024, 2*1024*1024, 4*1024*1024, 8*1024*1024, 16 *
             1024*1024, 32*1024*1024, 64*1024*1024, 128*1024*1024, 256*1024*1024, 512*1024*1024]

# (islocal, isfileref) pairs
locality_config = [(True, True), (True, False), (False, True), (False, False)]
num_experiments_per_config = 5

base_request_object = {"request_id": "", "pass_data": False, "data": "", "filename": "", "size": 167772160, "loops": 16, "terminal_service_number": 2,
                       "source_service_number": 0, "source_service_ip": "", "source_service_port": 0, "frontend_ip": "", "service_ips": ""}

local_url = "http://host1:82/service/0/start"
remote_url = "http://host2:82/service/0/start"


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


req_number = start_req_id - 1
for datasize in filesizes:
    for (islocal, isfileref) in locality_config:
        for i in range(num_experiments_per_config):
            req_number = req_number + 1
            req_str = make_request(
                str(req_number), datasize, islocal, isfileref)
            print(req_str)
            sys.stdout.flush()
            time.sleep(2)
