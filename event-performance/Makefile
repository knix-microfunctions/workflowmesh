-include proxy.mk
export TS ?= $(shell date +"%Y%m%d-%H%M%S")
export KO_DOCKER_REPO ?= localhost:5000

heartbeats:
	ko publish https://github.com/knix-microfunctions/workflowmesh/event-performance/cmd/heartbeats

function:
	ko publish https://github.com/knix-microfunctions/workflowmesh/event-performance/cmd/function

worker:
	ko publish https://github.com/knix-microfunctions/workflowmesh/event-performance/cmd/worker

move:
	./moveimc.sh

#TESTTIME=30
#SIZES=1024
TESTTIME=320
SIZES=1024 1048576 $(shell seq 2097152 2097152 41943040)

#############
# CHAIN test
#
# Uses Services that call one another in a chain.
chaintest-%.log:
	ko apply -f chain/chain_of_svc.yaml
	sleep 5
	SIZE=$$(echo $@|grep -o '[0-9]\+'); sed 's/value: "12345"/value: "'$${SIZE}'"/' -i chain/source.yaml
	ko apply -f chain/source.yaml
	sleep $(TESTTIME)
	ko delete -f chain/source.yaml
	sleep 2
	kubectl logs $$(kubectl get pod|grep '^step5'|sed 's/\s.*//'|sort|tail -n1) -c user-container >$@; 
	ko delete -f chain/chain_of_svc.yaml
	SIZE=$$(echo $@|grep -o '[0-9]\+'); sed 's/value: "'$${SIZE}'"/value: "12345"/' -i chain/source.yaml

chaintest-%.csv: chaintest-%.log
	SIZE=$$(echo $@|grep -o '[0-9]\+'); grep -o '\[.*\]' $^|tr ' ' ','|sed 's/\[/'$$SIZE',/'|sed 's/\]//'>$@
	sed -i '1d' $@

CHAINTESTFILES=$(foreach var,$(SIZES),chaintest-$(var).csv)
chaintest-totals.csv: $(CHAINTESTFILES)
	#cat $^ >$@
	python eval.py chain
	sed -i 's/\./,/g' chaintest-totals.csv 

chaintest-results.txt: \
		chaintest-1024.csv \
		chaintest-2048.csv \
		chaintest-4096.csv \
		chaintest-8192.csv \
		chaintest-16384.csv \
		chaintest-32768.csv \
		chaintest-65536.csv \
		chaintest-131072.csv \
		chaintest-262144.csv \
		chaintest-524288.csv \
		chaintest-1048576.csv \
		chaintest-2097152.csv \
		chaintest-4194304.csv \
		chaintest-8388608.csv \
		chaintest-16777216.csv \
		chaintest-33554432.csv \
		chaintest-67108864.csv \
		chaintest-134217728.csv \
		chaintest-268435456.csv
	cat $^ >$@

#############
# KCHAIN test
#
# Uses Knative Serving Services that call one another in a chain. Knative 
# serving does the forwarding/load balancing through the ingress gateway 
# (Istio in our case)
kchaintest-%.log:
	ko apply -f chain/chain_of_ksvc.yaml
	sleep 5
	SIZE=$$(echo $@|grep -o '[0-9]\+'); sed 's/value: "12345"/value: "'$${SIZE}'"/' -i chain/source.yaml
	ko apply -f chain/source.yaml
	sleep $(TESTTIME)
	ko delete -f chain/source.yaml
	sleep 2
	kubectl logs $$(kubectl get pod|grep '^step5'|sed 's/\s.*//'|sort|tail -n1) -c user-container >$@; 
	ko delete -f chain/chain_of_ksvc.yaml
	SIZE=$$(echo $@|grep -o '[0-9]\+'); sed 's/value: "'$${SIZE}'"/value: "12345"/' -i chain/source.yaml

kchaintest-%.csv: kchaintest-%.log
	SIZE=$$(echo $@|grep -o '[0-9]\+'); grep -o '\[.*\]' $^|tr ' ' ','|sed 's/\[/'$$SIZE',/'|sed 's/\]//'>$@
	sed -i '1d' $@

KCHAINTESTFILES=$(foreach var,$(SIZES),kchaintest-$(var).csv)
kchaintest-totals.csv: $(KCHAINTESTFILES)
	#cat $^ >$@
	python eval.py kchain
	sed -i 's/\./,/g' kchaintest-totals.csv 

################
# SEQUENCE test
#
# Uses Knative Eventing Sequence Services that are called one by one by the 
# sequence (set of channels). We're using the IMC Channel Dispatcher for this
seqtest-%.log:
	ko apply -f sequence/seq-of-svc.yaml
	sleep 5
	SIZE=$$(echo $@|grep -o '[0-9]\+'); sed 's/value: "12345"/value: "'$${SIZE}'"/' -i sequence/source.yaml
	ko apply -f sequence/source.yaml
	sleep $(TESTTIME)
	ko delete -f sequence/source.yaml
	sleep 2
	kubectl logs $$(kubectl get pod|grep '^step5'|sed 's/\s.*//'|sort|tail -n1) -c user-container >$@; 
	ko delete -f sequence/seq-of-svc.yaml
	SIZE=$$(echo $@|grep -o '[0-9]\+'); sed 's/value: "'$${SIZE}'"/value: "12345"/' -i sequence/source.yaml

seqtest-%.csv: seqtest-%.log
	SIZE=$$(echo $@|grep -o '[0-9]\+'); grep -o '\[.*\]' $^|tr ' ' ','|sed 's/\[/'$$SIZE',/'|sed 's/\]//'>$@
	sed -i '1d' $@

SEQTESTFILES=$(foreach var,$(SIZES),seqtest-$(var).csv)
seqtest-totals.csv: $(SEQTESTFILES)
	#cat $^ >$@
	python eval.py seq
	sed -i 's/\./,/g' seqtest-totals.csv 

################
# KSEQUENCE test
#
# Uses Knative Eventing Sequence of _Knative Services_ that are called one by 
# one by the sequence (set of channels). We're using the IMC Channel 
# Dispatcher for this

kseqtest-%.log:
	ko apply -f sequence/seq-of-ksvc.yaml
	sleep 5
	SIZE=$$(echo $@|grep -o '[0-9]\+'); sed 's/value: "12345"/value: "'$${SIZE}'"/' -i sequence/source.yaml
	ko apply -f sequence/source.yaml
	sleep $(TESTTIME)
	ko delete -f sequence/source.yaml
	sleep 2
	kubectl logs $$(kubectl get pod|grep '^step5'|sed 's/\s.*//'|sort|tail -n1) -c user-container >$@; 
	ko delete -f sequence/seq-of-ksvc.yaml
	SIZE=$$(echo $@|grep -o '[0-9]\+'); sed 's/value: "'$${SIZE}'"/value: "12345"/' -i sequence/source.yaml

kseqtest-%.csv: kseqtest-%.log
	SIZE=$$(echo $@|grep -o '[0-9]\+'); grep -o '\[.*\]' $^|tr ' ' ','|sed 's/\[/'$$SIZE',/'|sed 's/\]//'>$@
	sed -i '1d' $@

KSEQTESTFILES=$(foreach var,$(SIZES),kseqtest-$(var).csv)
kseqtest-totals.csv: $(KSEQTESTFILES)
	#cat $^ >$@
	python eval.py kseq
	sed -i 's/\./,/g' kseqtest-totals.csv 
