use axum::{extract::State, Json};

use crate::{
    config::{PlantConfig},
    model::LastSeenResponse,
    GlobalState,
};

pub async fn last_seen(state: State<GlobalState>) -> Json<Option<LastSeenResponse>> {
    let state_res = state.state.get();
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
