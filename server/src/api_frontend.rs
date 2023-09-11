use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    config::PlantConfig,
    model::{LastSeenResponse, WateringJob},
    state::StateError,
    GlobalState,
};

pub async fn last_seen(state: State<GlobalState>) -> Json<Option<LastSeenResponse>> {
    let state_res = state.json_state.get();
    if let Err(e) = state_res {
        log::error!("Error reading state: {:?}", e);
        return Json(None);
    }
    let state = state_res.unwrap();
    let last_seen_response = LastSeenResponse {
        last_seen_timestamp: state.last_seen.timestamp(),
        last_battery_percentage: state.last_accu_percentage,
    };
    Json(Some(last_seen_response))
}

pub async fn get_plant(state: State<GlobalState>) -> Json<Vec<PlantConfig>> {
    let plants = state.config.get_plant_config().unwrap();
    Json(plants)
}

#[derive(Deserialize, Debug)]
pub struct SetAmountMlQuery {
    amount_ml: usize,
}

pub async fn set_plant_amount_ml(
    state: State<GlobalState>,
    Path(name): Path<String>,
    Query(SetAmountMlQuery { amount_ml }): Query<SetAmountMlQuery>,
) -> (StatusCode, String) {
    let plants = state.config.get_plant_config();
    if let Err(err) = plants {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error reading config: {}", err),
        );
    }
    let plants = plants.unwrap();
    let plant = plants
        .iter()
        .enumerate()
        .find(|p| p.1.name == name);
    match plant {
        Some(plant) => {
            match state.config.put_plant_amount_ml(plant.0, amount_ml as u32) {
                Ok(_) => (StatusCode::OK, format!("Plant {} now gets {}ml/day", name, amount_ml)),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error saving config: {}", e)),
            }
        }
        None => (StatusCode::NOT_FOUND, format!("Plant {} not found", name))
    }
}

pub async fn test_watering(
    state: State<GlobalState>,
    plantname: Path<String>,
) -> (StatusCode, String) {
    let plants = state.config.get_plant_config().unwrap();
    let plant_index = plants
        .iter()
        .enumerate()
        .find(|(_, c)| c.name == plantname.as_ref());
    if plant_index.is_none() {
        return (StatusCode::BAD_REQUEST, "Plant not found".into());
    }
    let plant_index = plant_index.unwrap();
    let watering_job = WateringJob {
        plant_index: plant_index.0,
        duration_ms: plant_index.1.amount_ml as usize,
    };
    let ack = state.pending_warting_test.set_pending_job(watering_job);
    match ack.await.await {
        Err(_) => (
            StatusCode::GONE,
            "Another testing job has been started".into(),
        ),
        Ok(_) => (
            StatusCode::OK,
            format!("Plant {} should have been watered", *plantname),
        ),
    }
}
