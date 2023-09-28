use embedded_svc::http::client::*;
use embedded_svc::utils::io;
use esp_idf_svc::http::client::*;
use serde::Deserialize;

// TODO: resistance against trailing slash
// e.g. https://myserver.dev/evergreen/api, no tailing slash
const BASE_URL: &str = env!("API_BASE_URL");
const API_SECRET: &str = env!("API_SECRET");

#[derive(Debug)]
pub enum QueryError {
    Connection,         // HTTP, TLS
    UnexpectedResponse, // mal formatted Json, 404, unexpected format
}

// Copied from server side
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerWateringJob {
    pub plant_index: usize,
    pub amount_ml: usize,
}

// Copied from server side
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DequeueJobs {
    pub watering_jobs: Vec<ServerWateringJob>,
    pub sleep_recommendation_seconds: u64,
}

pub fn fetch_jobs(accu_percentage: f32) -> Result<DequeueJobs, QueryError> {
    let mut client = Client::wrap(
        EspHttpConnection::new(&Configuration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            ..Default::default()
        })
        .unwrap(),
    );

    // 10KiB make overflow
    let mut buffer = [0_u8; 128];

    // Get Jobs
    let url = format!(
        "{}/dequeue_jobs?accu_percentage={}&api_secret={}",
        BASE_URL, accu_percentage, API_SECRET
    );
    let request = client.post(&url, &[]).unwrap();
    let mut response = request.submit().unwrap();
    let read = io::try_read_full(&mut response, &mut buffer).map_err(|_| QueryError::Connection)?;
    println!("Bytes read1: {}", read);
    let body = String::from_utf8_lossy(&buffer[..read]).into_owned();
    println!("Response1: {}", body);

    let server_jobs: DequeueJobs =
        serde_json::from_str(&body).map_err(|_| QueryError::UnexpectedResponse)?;

    Ok(server_jobs)
}
