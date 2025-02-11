show-tables:
    just query_hub show tables

podman-ps:
    podman ps

podman-run-influxdb3:
    podman run -d --name influxdb3 -p 43121:8181 \
        quay.io/influxdb/influxdb3-core:latest \
        serve --node-id=local01 --object-store=memory

podman-run-grafana:
    podman run -d -p 43122:3000 --name grafana grafana/grafana-enterprise

influxdb *ARGS:
    podman exec -it influxdb3 influxdb3 {{ARGS}}

default_db := "hub"

create-database db=default_db:
    just influxdb create database {{db}}

query db *ARGS:
    just influxdb query --database {{db}} '"{{ARGS}}"'

query_hub *ARGS:
    just query hub {{ARGS}}

show-grafana-tips:
    # add data source for influxdb
    # URL -> http://host.docker.internal:43121
    # database -> hub
    # insecure connection -> on

install-wash:
    cargo install --locked wash-cli

wash-ui:
    # need to run v0.5.0, the latest (v0.6.0) is not working
    wash ui --experimental --version v0.5.0
