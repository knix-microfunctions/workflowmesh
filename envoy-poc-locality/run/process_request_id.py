import sys


def process_request_id_timings(log_file, request_id, islocal, isfileref):
    loglines = []
    with open(log_file, "r") as f:
        loglines = f.readlines()

    loglines_request_id = []
    request_id_str = f"[{request_id}]"
    for line in loglines:
        if line.find(request_id_str) >= 0:
            loglines_request_id.append(line)

    request_start_time = get_time(
        loglines_request_id, "[REQUEST_START_FRONTEND]")
    request_end_time = get_time(loglines_request_id, "[REQUEST_END_FRONTEND]")
    assert(request_start_time > 0.0)
    assert(request_end_time > 0.0)
    assert(request_end_time > request_start_time)
    request_time_diff = request_end_time - request_start_time

    data_process_start_time = get_time(
        loglines_request_id, "[DATA_PROCESS_START]")
    data_process_end_time = get_time(loglines_request_id, "[DATA_PROCESS_END]")
    assert(data_process_start_time > 0.0)
    assert(data_process_end_time > 0.0)
    assert(data_process_end_time > data_process_start_time)
    data_process_time_diff = data_process_end_time - data_process_start_time

    service_call_start_time = get_time(
        loglines_request_id, "[CALL_SERVICE_START]")
    service_call_end_time = get_time(
        loglines_request_id, "[CALL_SERVICE_END]")
    assert(service_call_start_time > 0.0)
    assert(service_call_end_time > 0.0)
    assert(service_call_end_time > service_call_start_time)
    service_call_time_diff = service_call_end_time - service_call_start_time

    file_read_start_time = 0.0
    file_read_time_diff = 0.0
    if isfileref == 1:
        file_read_start_time = get_time(
            loglines_request_id, "[FILE_READ_START]")
        assert(file_read_start_time > 0.0)
        assert(data_process_start_time > file_read_start_time)
        file_read_time_diff = data_process_start_time - file_read_start_time
        assert(file_read_time_diff > 0.0)

    assert((request_time_diff - (data_process_time_diff +
                                 service_call_time_diff + file_read_time_diff)) >= 0.0)
    return request_time_diff, data_process_time_diff, service_call_time_diff, file_read_time_diff


def get_time(loglines_request_id, pattern):
    line_of_interest = ""
    for line in loglines_request_id:
        if line.find(pattern) >= 0:
            line_of_interest = line.strip()

    line_of_interest = line_of_interest.split(" ")
    time_str = line_of_interest[2]
    time_str = time_str.replace("[", "")
    time_str = time_str.replace("]", "")
    time_str = time_str.strip()
    return float(time_str)


log_file = sys.argv[1]
request_id_str = sys.argv[2]
request_size_str = sys.argv[3]
islocal_str = sys.argv[4]
isfileref_str = sys.argv[5]

request_id = int(request_id_str)
request_size = int(request_size_str)
islocal = int(islocal_str)
isfileref = int(isfileref_str)

request_time_diff, data_process_time_diff, service_call_time_diff, file_read_time_diff = process_request_id_timings(
    log_file, request_id, islocal, isfileref)
prefix = f"{request_id},{request_size},{islocal},{isfileref},"
print(prefix + "%.3f,%.3f,%.3f,%.3f" % (request_time_diff,
                                        data_process_time_diff, service_call_time_diff, file_read_time_diff))
