podman-ps:
    podman ps

podman-run-influxdb3:
    podman run --name influxdb3 -p 43121:8181 \
        quay.io/influxdb/influxdb3-core:latest \
        serve --node-id=local01 --object-store=memory

podman-run-grafana:
    podman run -p 43122:3000 --name grafana grafana/grafana-enterprise

influxdb3 *ARGS:
    podman exec influxdb3 influxdb3 {{ARGS}}

default_db := "hub"

create-database db=default_db:
    just influxdb3 create database {{db}}

show-grafana-tips:
    # add data source for influxdb
    # URL -> http://host.docker.internal:43121
    # database -> hub
    # insecure connection -> on
