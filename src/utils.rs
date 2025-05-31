use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

pub fn read_metrics_text(url: &str) -> Result<String, Box<dyn Error>> {
    if url.starts_with("http://") || url.starts_with("https://") {
        let mut response = ureq::get(url)
            .config()
            .timeout_global(Some(Duration::from_secs(15)))
            .build()
            .call()?;
        if response.status().is_success() {
            response.body_mut().read_to_string().map_err(|e| e.into())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to fetch metrics text with status: {}",
                    response.status().as_str()
                ),
            )))
        }
    } else {
        let metrics_file_path = PathBuf::from(url);
        if metrics_file_path.exists() && metrics_file_path.is_file() {
            std::fs::read_to_string(metrics_file_path).map_err(|e| e.into())
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Metrics file not found: {}", url),
            )))
        }
    }
}

pub fn extract_source(url_or_path: &str) -> String {
    if url_or_path.starts_with("http://") || url_or_path.starts_with("https://") {
        // If the source is not provided, extract endpoint from the URL
        let offset = url_or_path.find("://").unwrap_or(0) + 3; // Skip protocol part
        let end = url_or_path[offset..]
            .find('/')
            .map_or(url_or_path.len(), |pos| offset + pos);
        url_or_path[offset..end].to_string()
    } else {
        url_or_path.to_string()
    }
}
