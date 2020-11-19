#export http_proxy ?= http://<your-proxy>:<port>
#export https_proxy ?= http://<your-proxy>:<port>
#export no_proxy ?= <noproxy>
export PATH = $(shell echo $$PATH:$$GOPATH/bin)