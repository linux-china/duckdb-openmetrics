use crate::utils::read_metrics_text;
use duckdb::core::{DataChunkHandle, Inserter, LogicalTypeHandle, LogicalTypeId};
use duckdb::vtab::{BindInfo, InitInfo, TableFunctionInfo, VTab};
use openmetrics_parser::prometheus::parse_prometheus;
use openmetrics_parser::PrometheusValue;
use std::error::Error;
use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

#[repr(C)]
pub struct PrometheusParams {
    pub url: String,
    pub source: Option<String>,
}

#[repr(C)]
pub struct PrometheusInitData {
    pub done: AtomicBool,
}

pub struct PrometheusVTab;

impl VTab for PrometheusVTab {
    type InitData = PrometheusInitData;
    type BindData = PrometheusParams;

    fn bind(bind: &BindInfo) -> Result<Self::BindData, Box<dyn Error>> {
        // columns: name, metric_type, value, labels, source, created_at, details
        bind.add_result_column(
            "metric_name",
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        );
        bind.add_result_column(
            "metric_type",
            LogicalTypeHandle::from(LogicalTypeId::Varchar),
        );
        bind.add_result_column("value", LogicalTypeHandle::from(LogicalTypeId::Double));
        bind.add_result_column("unit", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column("labels", LogicalTypeHandle::from(LogicalTypeId::Varchar)); // json text
        bind.add_result_column("source", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        bind.add_result_column(
            "timestamp",
            LogicalTypeHandle::from(LogicalTypeId::Timestamp),
        );
        // json text for histogram and summary
        bind.add_result_column("details", LogicalTypeHandle::from(LogicalTypeId::Varchar));
        let url = bind.get_parameter(0).to_string();
        let mut source = bind.get_parameter(1).to_string();
        if source.is_empty() {
            // If the source is not provided, extract endpoint from the URL
            let offset = url.find("://").unwrap_or(0) + 3; // Skip protocol part
            let end = url[offset..]
                .find('/')
                .map_or(url.len(), |pos| offset + pos);
            source = url[offset..end].to_string();
        }
        Ok(PrometheusParams {
            url,
            source: Some(source),
        })
    }

    fn init(_: &InitInfo) -> Result<Self::InitData, Box<dyn Error>> {
        Ok(PrometheusInitData {
            done: AtomicBool::new(false),
        })
    }

    fn func(
        func: &TableFunctionInfo<Self>,
        output: &mut DataChunkHandle,
    ) -> Result<(), Box<dyn Error>> {
        let init_data = func.get_init_data();
        let bind_data = func.get_bind_data();
        if init_data.done.swap(true, Ordering::Relaxed) {
            output.set_len(0);
        } else {
            let now = SystemTime::now();
            let micro_seconds = now
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                * 1000;
            let http_url = bind_data.url.as_str();
            let source = &bind_data.source.clone().unwrap_or("".to_owned());
            let vector_name = output.flat_vector(0);
            let vector_metric_type = output.flat_vector(1);
            let mut vector_value = output.flat_vector(2);
            let slice_value = vector_value.as_mut_slice::<f64>();
            let vector_unit = output.flat_vector(3);
            let vector_labels = output.flat_vector(4);
            let vector_source = output.flat_vector(5);
            let mut vector_created_at = output.flat_vector(6);
            let vector_detail = output.flat_vector(7);
            let slice_created_at = vector_created_at.as_mut_slice::<u64>();
            match read_metrics_text(http_url) {
                Ok(metrics_text) => {
                    if let Ok(exposition) = parse_prometheus(&metrics_text) {
                        let mut sample_index = 0;
                        for (metric_name, family) in &exposition.families {
                            // A MetricFamily is a collection of metrics with the same type, name
                            let metric_type = &family.family_type.to_string();
                            let unit = family.unit.as_str();
                            for sample in family.iter_samples() {
                                vector_name
                                    .insert(sample_index, CString::new(metric_name.as_str())?);
                                vector_metric_type
                                    .insert(sample_index, CString::new(metric_type.as_str())?);
                                vector_unit.insert(sample_index, CString::new(unit)?);
                                vector_source.insert(sample_index, CString::new(source.as_str())?);
                                let timestamp = sample
                                    .timestamp
                                    .map(|timestamp| timestamp as u64)
                                    .unwrap_or(micro_seconds);
                                slice_created_at[sample_index] = timestamp;
                                if let Ok(labels) = sample.get_labelset() {
                                    let labels_count = labels.iter().count();
                                    if labels_count > 0 {
                                        let body = labels
                                            .iter()
                                            .map(|(key, value)| format!(r#""{}":"{}""#, key, value))
                                            .collect::<Vec<_>>()
                                            .join(",");
                                        let labels_json = format!("{{{}}}", body);
                                        vector_labels
                                            .insert(sample_index, CString::new(labels_json)?);
                                    }
                                }
                                match &sample.value {
                                    PrometheusValue::Unknown(value) => {
                                        slice_value[sample_index] = value.as_f64();
                                    }
                                    PrometheusValue::Untyped(value) => {
                                        slice_value[sample_index] = value.as_f64();
                                    }
                                    PrometheusValue::Gauge(gauge) => {
                                        slice_value[sample_index] = gauge.as_f64();
                                    }
                                    PrometheusValue::Counter(counter) => {
                                        slice_value[sample_index] = counter.value.as_f64();
                                    }
                                    PrometheusValue::Histogram(histogram) => {
                                        let histogram_count = histogram.count.unwrap_or(0);
                                        let histogram_sum =
                                            histogram.sum.map(|sum| sum.as_f64()).unwrap_or(0.0);
                                        if histogram_count > 0 {
                                            slice_value[sample_index] =
                                                histogram_sum / histogram_count as f64;
                                        } else {
                                            slice_value[sample_index] = 0.0;
                                        }
                                        let details = if histogram.buckets.is_empty() {
                                            format!(
                                                r#"{{"count":{}, "sum":{}}}"#,
                                                histogram_count, histogram_sum
                                            )
                                        } else {
                                            let buckets_text = histogram
                                                .buckets
                                                .iter()
                                                .map(|bucket| {
                                                    let upper_bound = bucket.upper_bound;
                                                    let count = bucket.count.as_f64();
                                                    format!(
                                                        r#"{{"upper_bound":{}, "count":{}}}"#,
                                                        upper_bound, count
                                                    )
                                                })
                                                .collect::<Vec<_>>()
                                                .join(",");
                                            format!(
                                                r#"{{"count":{}, "sum":{}, "buckets":[{}]}}"#,
                                                histogram_count, histogram_sum, buckets_text
                                            )
                                        };
                                        vector_detail.insert(sample_index, CString::new(details)?);
                                    }
                                    PrometheusValue::Summary(summary) => {
                                        let summary_count = summary.count.unwrap_or(0);
                                        let summary_sum =
                                            summary.sum.map(|sum| sum.as_f64()).unwrap_or(0.0);
                                        if summary_count > 0 {
                                            slice_value[sample_index] =
                                                summary_sum / summary_count as f64;
                                        } else {
                                            slice_value[sample_index] = 0.0;
                                        }
                                        let details = if summary.quantiles.is_empty() {
                                            format!(
                                                r#"{{"count":{}, "sum":{}}}"#,
                                                summary_count, summary_sum
                                            )
                                        } else {
                                            let quantiles_text = summary
                                                .quantiles
                                                .iter()
                                                .map(|quantile| {
                                                    let upper_bound = quantile.quantile;
                                                    let value = quantile.value.as_f64();
                                                    format!(
                                                        r#"{{"quantile":{}, "value":{}}}"#,
                                                        upper_bound, value
                                                    )
                                                })
                                                .collect::<Vec<_>>()
                                                .join(",");
                                            format!(
                                                r#"{{"count":{}, "sum":{}, "quantiles":[{}]}}"#,
                                                summary_count, summary_sum, quantiles_text
                                            )
                                        };
                                        vector_detail.insert(sample_index, CString::new(details)?);
                                    }
                                }
                                sample_index = sample_index + 1;
                            }
                        }
                        output.set_len(sample_index);
                    } else {
                        output.set_len(0);
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to parse Prometheus metrics text",
                        )));
                    }
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    fn parameters() -> Option<Vec<LogicalTypeHandle>> {
        let handle_url = LogicalTypeHandle::from(LogicalTypeId::Varchar);
        let handle_source = LogicalTypeHandle::from(LogicalTypeId::Varchar);
        Some(vec![handle_url, handle_source])
    }
}
