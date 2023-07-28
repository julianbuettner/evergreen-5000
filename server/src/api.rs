use axum::extract::State;

use crate::{stateholder::Stateholder, model::LastSeenResponse};


pub async fn last_seen(holder: State<Stateholder>) -> Option<LastSeenResponse> {
    let state_res = holder.get();
    if let Err(e) = state_res {
        log::error!("Error reading state: {:?}", e);
        return None;
    }
    let state = state_res.unwrap();
    let last_seen_response = LastSeenResponse {
        last_seen_timestamp: state.last_seen.timestamp(),
        last_battery_percentage: state.last_accu_percentage,
    };
    Some(last_seen_response)
}
