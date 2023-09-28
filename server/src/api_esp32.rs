use std::time::Duration;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use axum_client_ip::SecureClientIp;
use chrono::{Duration as ChronoDuration, Local, NaiveDate};
use log::{error, warn};
use serde::Deserialize;

use crate::{
    model::{DequeueJobs, WateringJob},
    GlobalState,
};

fn next_watering_sleep_time(last_date: NaiveDate) -> ChronoDuration {
    // ChronoDuration can be negative
    let now = Local::now().naive_local();
    let last_watering = last_date.and_hms_opt(9, 0, 0).expect("Invalid hms");
    let h24 = ChronoDuration::from_std(Duration::from_secs(24 * 60 * 60)).unwrap();
    let next_watering = last_watering + h24;

    // 9h - 10h on same day => -1h to "wait"
    next_watering - now
}

#[derive(Deserialize, Debug)]
pub struct DequeueQuery {
    accu_percentage: f32,
    // Api call defines the allowed IP, so it must be protected.
    api_secret: String,
}

pub async fn dequeue_jobs(
    state: State<GlobalState>,
    Query(query): Query<DequeueQuery>,
    SecureClientIp(ip): SecureClientIp,
) -> (StatusCode, Result<Json<DequeueJobs>, String>) {
    let expected_secret = state.config.get_api_secret();
    if let Err(e) = expected_secret {
        error!("Could not read API secret: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Err("Error reading confi".into()),
        );
    }
    let expected_secret = expected_secret.unwrap();
    if expected_secret != query.api_secret {
        // I know, one does not log wrong passwords, but
        // it's not a real password and it's helpful.
        warn!("Provided API secret \"{}\" was wrong", query.api_secret);
        return (StatusCode::UNAUTHORIZED, Err("Wrong API secret".into()));
    }

    println!(
        "ESP32 with IP {} reports: Accu: {}",
        ip, query.accu_percentage
    );
    // If watering test is pending, do just that one
    if let Some(task) = state.pending_warting_test.pop_pending_task().await {
        let test_job = DequeueJobs {
            watering_jobs: vec![task.destruct_and_ack()],
            sleep_recommendation_seconds: 0,
        };
        return (StatusCode::OK, Ok(Json(test_job)));
    }

    // Check if watering should happen now
    let json_state = state.json_state.ensure_state();
    if let Err(err) = json_state {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Err(format!("Error reading json state: {}", err)),
        );
    }
    let mut json_state = json_state.unwrap();
    let plant_config = state.config.get_plant_config();
    if let Err(err) = plant_config {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Err(format!("Error reading plant config: {}", err)),
        );
    }
    let plant_config = plant_config.unwrap();
    let now = Local::now().naive_local();
    let watering_sleep_time_sec: i64 =
        next_watering_sleep_time(json_state.last_planned_watering).num_seconds();
    let jobs: Vec<WateringJob> = match watering_sleep_time_sec.is_negative() {
        true => {
            json_state.last_planned_watering = now.date();
            plant_config
                .iter()
                .enumerate()
                .map(|(index, conf)| WateringJob {
                    plant_index: index,
                    amount_ml: conf.amount_ml,
                })
                .collect()
        }
        false => Vec::new(),
    };
    json_state.last_seen = Local::now().naive_local();
    json_state.last_ip = ip;
    json_state.last_accu_percentage = query.accu_percentage;
    if let Err(err) = state.json_state.set(json_state.clone()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Err(format!("Json Store error: {}", err)),
        );
    }

    let sleep_recommendation = next_watering_sleep_time(json_state.last_planned_watering);
    let sleep_recommendation_seconds = if sleep_recommendation.num_seconds().is_negative() {
        0
    } else {
        sleep_recommendation.num_seconds() as u64
    };
    let waterig_job = DequeueJobs {
        watering_jobs: jobs,
        sleep_recommendation_seconds,
    };
    (StatusCode::OK, Ok(Json(waterig_job)))
}
