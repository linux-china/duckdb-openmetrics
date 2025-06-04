# build extension with debug mode
build:
    make debug

# build extension with release mode
release:
    make release

# build(release) and install extension to DuckDB
install: release
    ls -al build/release/extension/openmetrics/openmetrics.duckdb_extension
    cp -rf build/release/extension/openmetrics/openmetrics.duckdb_extension ~/.duckdb/extensions/v1.3.0/osx_amd64

# run DuckDB to get prometheus metrics
run-prometheus: install
    duckdb -unsigned -cmd "install openmetrics; load openmetrics;" -s "select * from prometheus('tests/actuator-prometheus.txt','');"

# run DuckDB to get openmetrics metrics
run-openmetrics: install
    duckdb -unsigned -cmd "install openmetrics; load openmetrics;" -s "select * from openmetrics('tests/openmetrics.txt','');"

# configure build system
configure:
    make configure

# pull submodules
pull-submodules:
    git submodule update --init --recursive --depth=1
