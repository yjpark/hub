podman-ps:
    podman ps
podman-run-influxdb3:
    podman run --name influxdb3 -p 43121:8181 \
        quay.io/influxdb/influxdb3-core:latest \
        serve --node-id=local01 --object-store=memory
