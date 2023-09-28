use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum_client_ip::SecureClientIp;
use log::{error, info, warn};
use serde::Deserialize;

use crate::{
    config::PlantConfig,
    model::{LastSeenResponse, WateringJob},
    GlobalState,
};

pub async fn last_seen(state: State<GlobalState>) -> Json<Option<LastSeenResponse>> {
    let state_res = state.json_state.get();
    if let Err(e) = state_res {
        log::error!("Error reading state: {:?}", e);
        return Json(None);
    }
    let state = state_res.unwrap();
    info!("Last seen request - ESP32 last seen: {}", state.last_seen);
    let last_seen_response = LastSeenResponse {
        last_seen_timestamp: state.last_seen.timestamp(),
        last_battery_percentage: state.last_accu_percentage,
        last_watering_date: state.last_planned_watering.to_string(),
    };
    Json(Some(last_seen_response))
}

pub async fn get_plant(state: State<GlobalState>) -> Json<Vec<PlantConfig>> {
    let plants = state.config.get_plant_config().unwrap();
    info!("Get plants request - plant count {}", plants.len());
    Json(plants)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetAmountMlQuery {
    amount_ml: usize,
}

pub async fn set_plant_amount_ml(
    state: State<GlobalState>,
    Path(name): Path<String>,
    Query(SetAmountMlQuery { amount_ml }): Query<SetAmountMlQuery>,
) -> (StatusCode, String) {
    info!("Setting plant amount to {}ml", amount_ml);

    if amount_ml > 500 {
        warn!(
            "Request to water {}ml > 500ml received. Declined.",
            amount_ml
        );
        return (
            StatusCode::BAD_REQUEST,
            "More then 500ml are not allowed".to_string(),
        );
    }

    let plants = state.config.get_plant_config();
    if let Err(err) = plants {
        error!("Error reading plant config");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error reading config: {}", err),
        );
    }
    let plants = plants.unwrap();
    let plant = plants.iter().enumerate().find(|p| p.1.name == name);
    match plant {
        Some(plant) => match state.config.put_plant_amount_ml(plant.0, amount_ml as u32) {
            Ok(_) => (
                StatusCode::OK,
                format!("Plant {} now gets {}ml/day", name, amount_ml),
            ),
            Err(e) => {
                error!("Error saving config for amount update: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error saving config: {}", e),
                )
            }
        },
        None => (StatusCode::NOT_FOUND, format!("Plant {} not found", name)),
    }
}

pub async fn test_watering(
    state: State<GlobalState>,
    plantname: Path<String>,
    SecureClientIp(ip): SecureClientIp,
) -> (StatusCode, String) {
    info!("Starting watering test...");
    let esp32_ip = state.json_state.ensure_state().map(|s| s.last_ip);
    if let Err(e) = esp32_ip {
        error!("Could not read json state: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not read json state".into(),
        );
    }
    let esp32_ip = esp32_ip.unwrap();

    if ip != esp32_ip {
        warn!("IP mismatch. Frontend: {}, ESP32: {}", ip, esp32_ip);
        return (StatusCode::FORBIDDEN, format!("Your IP {} must be the same as ESP32's", ip));
    }

    let plants = state.config.get_plant_config().unwrap();
    let plant_index = plants
        .iter()
        .enumerate()
        .find(|(_, c)| c.name == plantname.as_ref());
    if plant_index.is_none() {
        info!("Plant {} not found", plantname.to_string());
        return (StatusCode::BAD_REQUEST, "Plant not found".into());
    }
    let plant_index = plant_index.unwrap();
    let watering_job = WateringJob {
        plant_index: plant_index.0,
        amount_ml: plant_index.1.amount_ml,
    };
    let ack = state.pending_warting_test.set_pending_job(watering_job);
    info!("Waiting now for the job to be picked up...");
    match ack.await.await {
        Err(_) => {
            info!("Aborted test - another test startet");
            (
                StatusCode::GONE,
                "Another testing job has been started".into(),
            )
        }
        Ok(_) => {
            info!("ESP32 dequeued the job!");
            (
                StatusCode::OK,
                format!("Plant {} should have been watered", *plantname),
            )
        }
    }
}
