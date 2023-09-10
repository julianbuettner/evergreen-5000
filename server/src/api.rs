use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::{config::PlantConfig, model::LastSeenResponse, GlobalState};

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

pub async fn test_watering(
    state: State<GlobalState>,
    plantname: Path<String>,
) -> (StatusCode, String) {
    let plants = state.config.get_plant_config().unwrap();
    if plants
        .iter()
        .find(|c| c.name == plantname.as_ref())
        .is_none()
    {
        return (StatusCode::BAD_REQUEST, "Plant not found".into());
    }
    let ack = state
        .pending_warting_test
        .set_pending_job(plantname.to_string());
    match ack.await.await {
        Err(_) => (
            StatusCode::GONE,
            "Another testing job has been started".into(),
        ),
        Ok(Err(_)) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "ESP failed to water plant".into(),
        ),
        Ok(Ok(_)) => (
            StatusCode::OK,
            format!("Plant {} should have been watered", *plantname),
        ),
    }
}
