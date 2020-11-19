#!/bin/bash

# In order to

NODE1=node1
NODE2=node2

if [ "$1" != "" ]
then HN=$1
elif [ "$1" == "swap" ]
    HN=$(kubectl -n knative-eventing get deployment imc-dispatcher -o json|jq -r '.spec.template.spec.nodeSelector."kubernetes.io/hostname"')
    if [ "$HN" == "$NODE1" ]
    then 
        HN=$NODE1
    else
        HN=$NODE2
    fi
else
echo "Usage: $0 <kubernetes.io/hostname>|swap"
echo " moves the knative-eventing/imc-dispatcher to a specific host or alternates between $NODE1 and $NODE2"
fi
echo "Moving IMC dispatcher to $HN"
kubectl -n knative-eventing patch deployment imc-dispatcher --patch "spec:
  template:
    spec:
      nodeSelector:
        kubernetes.io/hostname: $HN"