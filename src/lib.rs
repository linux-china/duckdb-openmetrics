extern crate duckdb;
extern crate duckdb_loadable_macros;
extern crate libduckdb_sys;
mod open_metrics;
mod prometheus;
mod utils;

use duckdb::{Connection, Result};
use duckdb_loadable_macros::duckdb_entrypoint_c_api;
use libduckdb_sys::{self as ffi};

use crate::open_metrics::OpenMetricsVTab;
use crate::prometheus::PrometheusVTab;
use std::error::Error;

#[duckdb_entrypoint_c_api()]
pub unsafe fn extension_entrypoint(con: Connection) -> Result<(), Box<dyn Error>> {
    con.register_table_function::<PrometheusVTab>("prometheus")
        .expect("Failed to register prometheus table function");

    con.register_table_function::<OpenMetricsVTab>("openmetrics")
        .expect("Failed to register openmetrics table function");

    Ok(())
}
