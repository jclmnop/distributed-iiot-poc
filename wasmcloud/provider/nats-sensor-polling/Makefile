PROJECT = nats-sensor-polling
NAME = "nats-sensor-polling"
VENDOR = "jclmnop"
CAPABILITY_ID = wasmcloud:polling
VERSION  = $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[] .version' | head -1)
REVISION = 0
oci_url  = localhost:5000/v2/$(PROJECT):$(VERSION)

include provider.mk