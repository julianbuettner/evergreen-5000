use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{Local, NaiveDate, NaiveDateTime, Timelike};
use serde::Deserialize;

use crate::{
    duration_calculation::amount_ml_to_ms, model::WateringJob, state::JsonState, GlobalState,
};

// Water every day at 0900h.
// TODO: make configurable
fn watering_should_happen(last_date: NaiveDate) -> bool {
    let now = Local::now().naive_local();
    if now.date() == last_date {
        false
    } else {
        now.hour() >= 9
    }
}

#[derive(Deserialize, Debug)]
pub struct DequeueQuery {
    accu_percentage: f32,
}

pub async fn dequeue_jobs(
    state: State<GlobalState>,
    Query(query): Query<DequeueQuery>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> (StatusCode, Result<Json<Vec<WateringJob>>, String>) {
    println!("IP: {}, accu: {}", addr, query.accu_percentage);
    // If watering test is pending, do just that one
    if let Some(task) = state.pending_warting_test.pop_pending_task().await {
        return (StatusCode::OK, Ok(Json(vec![task.destruct_and_ack()])));
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
    let jobs: Vec<WateringJob> = match watering_should_happen(json_state.last_planned_watering) {
        true => {
            json_state.last_planned_watering = now.date();
            plant_config
                .iter()
                .enumerate()
                .map(|(index, conf)| WateringJob {
                    plant_index: index,
                    duration_ms: amount_ml_to_ms(conf.amount_ml),
                })
                .collect()
        }
        false => Vec::new(),
    };
    json_state.last_seen = Local::now().naive_local();
    json_state.last_accu_percentage = query.accu_percentage;
    if let Err(err) = state.json_state.set(json_state) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Err(format!("Json Store error: {}", err)),
        );
    }

    (StatusCode::OK, Ok(Json(jobs)))
}
