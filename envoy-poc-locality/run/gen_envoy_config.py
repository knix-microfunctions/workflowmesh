import re
import sys

input_file = sys.argv[1]
output_file = sys.argv[2]
local_envoy_port = sys.argv[3]
remote_envoy_host = sys.argv[4]
remote_envoy_port = sys.argv[5]
timeout = sys.argv[6]
print(f"input_file:{input_file}, output_file:{output_file}, local_envoy_port:{local_envoy_port}, remote_envoy_host:{remote_envoy_host}, remote_envoy_port:{remote_envoy_port}, timeout:{timeout}")
# read template file
with open(input_file) as f:
    text = f.read()
# replace templates with real values
text = re.sub('{{ LOCAL_ENVOY_PORT }}', local_envoy_port, text)
text = re.sub('{{ REMOTE_ENVOY_HOST }}', remote_envoy_host, text)
text = re.sub('{{ REMOTE_ENVOY_PORT }}', remote_envoy_port, text)
text = re.sub('{{ TIMEOUT }}', timeout, text)
# write config file
with open(output_file, 'w') as f:
    f.write(text)
