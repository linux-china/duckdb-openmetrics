DuckDB Prometheus/OpenMetrics extension
========================================

This is a DuckDB extension to query metrics data from Prometheus and OpenMetrics.

Features:

- Prometheus support: `prometheus(url_or_path, source)`
- OpenMetrics support: `openmetrics(url_or_path, source)`

# Usage

Prometheus metrics query:

```
$ duckdb -unsigned -cmd "install openmetrics; load openmetrics;"
duckdb> select * from prometheus('http://localhost:8888/actuator/prometheus','');
```

Function `prometheus(url_or_path, source)` and `openmetrics(url_or_path, source)` take two parameters:

- `url_or_path`: The URL or file path to the Prometheus or OpenMetrics endpoint.
- `source`: A string representing the source of the metrics. If `source` is empty(`''`), `source` will be endpoint(
  host+port) of url or file path.

# Metrics data table

Columns of metrics data:

- `metric_name`: The name of the metric.
- `metric_type`: The type of the metric (e.g., counter, gauges).
- `value`: The value of the metric as a floating-point number.
- `unit`: The unit of the metric, such as `seconds`, `bytes`, etc.
- `labels`: A JSON object containing the labels associated with the metric.
- `sources`: source of the metric, e.g., `127.0.0.1:8080`.
- `timestamp`: The timestamp of the metric in milliseconds since epoch.
- `details`: A JSON object containing additional details about the metric, such as summary percentiles and histogram
  buckets.

# Build and Install

- Get the source code: `git clone https://github.com/linux-china/duckdb-openmetrics.git`
- Change to the directory: `cd duckdb-openmetrics`
- Pull submodules: `git submodule update --init --recursive --depth=1`
- Configuration: `make configure`
- Build the extension: `make release`
- Install the extension: `make install`

### References

* Prometheus specification: https://prometheus.io/docs/concepts/data_model/
* OpenMetrics specification: https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md
* Template for Rust-based DuckDB
  extensions: https://github.com/prometheus/OpenMetrics/blob/main/specification/OpenMetrics.md
