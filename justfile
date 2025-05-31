# build extension with debug mode
build:
    make build

# build extension with release mode
release:
    make release

# build(release) and install extension to DuckDB
install: release
    ls -al build/release/extension/openmetrics/openmetrics.duckdb_extension
    cp -rf build/release/extension/openmetrics/openmetrics.duckdb_extension ~/.duckdb/extensions/v1.2.2/osx_amd64

# run DuckDB with the OpenMetrics extension
run-prometheus: install
    duckdb -unsigned -cmd "install openmetrics; load openmetrics;" -s "select * from prometheus('http://localhost:8888/actuator/prometheus','');"

# configure build system
configure:
    make configure

# pull submodules
pull-submodules:
    git submodule update --init --recursive --depth=1
