use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LastSeenResponse {
    pub last_seen_timestamp: i64,
    pub last_battery_percentage: f32,
}
