use openmetrics_parser::openmetrics::parse_openmetrics;
use openmetrics_parser::prometheus::parse_prometheus;
use std::time::Duration;

#[test]
fn test_parser() {
    println!("Hello, world!");
}

#[test]
fn test_ureq() {
    let http_url = "http://localhost:8888/actuator/prometheus";
    let response = ureq::get(http_url)
        .config()
        .timeout_global(Some(Duration::from_secs(15)))
        .build()
        .call()
        .expect(&format!("Failed to make request: {}", http_url))
        .body_mut()
        .read_to_string()
        .unwrap();
    println!("{}", response);
}

#[test]
fn test_parse_prometheus() {
    let text = include_str!("actuator-prometheus.txt");
    parse_prometheus(text).unwrap();
}

#[test]
fn test_openmetrics() {
    let text = include_str!("openmetrics.txt");
    parse_openmetrics(text).unwrap();
}
