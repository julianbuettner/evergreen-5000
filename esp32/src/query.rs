use std::time::Duration;

use embedded_svc::http::{self, client::*, status, Headers, Status};
use embedded_svc::io::Read;
use embedded_svc::utils::io;
use esp_idf_svc::http::client::*;
use serde::{Deserialize, Serialize};

// TODO: resistance against trailing slash
// e.g. https://myserver.dev/evergreen/api, no tailing slash
const BASE_URL: &str = env!("API_BASE_URL");
const API_SECRET: &str = env!("API_SECRET");

pub enum QueryError {
    Connection,         // HTTP, TLS
    UnexpectedResponse, // mal formatted Json, 404, unexpected format
}

pub struct Jobs {
    pub plantings: Vec<Duration>,
    pub sleep_recommendation: Duration,
}

// Copied from server side
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerWateringJob {
    pub plant_index: usize,
    pub duration_ms: usize,
}

fn server_jobs_to_jobs(server_jobs: Vec<ServerWateringJob>) -> Result<Vec<Duration>, QueryError> {
    // Maximum of 16 plants
    let mut waterings = [Duration::ZERO; 16];
    for server_job in server_jobs {
        if server_job.plant_index > waterings.len() {
            return Err(QueryError::UnexpectedResponse);
        }
        waterings[server_job.plant_index] = Duration::from_millis(server_job.duration_ms as u64);
    }
    Ok(waterings.to_vec())
}

pub fn fetch_jobs(accu_percentage: f32) -> Result<Jobs, QueryError> {
    let mut client = Client::wrap(
        EspHttpConnection::new(&Configuration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            ..Default::default()
        })
        .unwrap(),
    );

    // ESP has 520KiB RAM.
    let mut buffer = [0u8; 10 * 1024];

    // Get Jobs
    let url = format!("{}/dequeue_json", BASE_URL);
    let accu_percentage_string = accu_percentage.to_string();
    let headers = [("accu_percentage", accu_percentage_string.as_str())];
    let mut response = client.post(&url, &headers).unwrap().submit().unwrap();
    let read = io::try_read_full(&mut response, &mut buffer).map_err(|_| QueryError::Connection)?;
    let body = String::from_utf8_lossy(&buffer[..read]).into_owned();

    let server_jobs: Vec<ServerWateringJob> =
        serde_json::from_str(&body).map_err(|_| QueryError::UnexpectedResponse)?;

    let url = format!("{}/sleep_recommendation", BASE_URL);
    let mut response = client.get(&url).unwrap().submit().unwrap();
    let read = io::try_read_full(&mut response, &mut buffer).map_err(|_| QueryError::Connection)?;
    let body = String::from_utf8_lossy(&buffer[..read]).into_owned();

    let sleep_recommendation_sec: u64 = body.parse().map_err(|_| QueryError::UnexpectedResponse)?;

    Ok(Jobs {
        plantings: server_jobs_to_jobs(server_jobs)?,
        sleep_recommendation: Duration::from_secs(sleep_recommendation_sec),
    })
}

pub fn content() -> String {
    let url = String::from("https://whatthecommit.com/index.txt");

    println!("About to fetch content from {}", url);
    let mut client = Client::wrap(
        EspHttpConnection::new(&Configuration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            ..Default::default()
        })
        .unwrap(),
    );

    let mut response = client.get(&url).unwrap().submit().unwrap();
    let mut body = [0_u8; 3048];
    let read = io::try_read_full(&mut response, &mut body)
        .map_err(|err| err.0)
        .unwrap();

    let message = String::from_utf8_lossy(&body[..read]).into_owned();
    println!("Body (truncated to 3K):\n{:?}", message);

    // Complete the response
    while response.read(&mut body).unwrap() > 0 {}

    message
}
