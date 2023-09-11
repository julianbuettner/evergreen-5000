use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct LastSeenResponse {
    pub last_seen_timestamp: i64,
    pub last_battery_percentage: f32,
}

#[derive(Debug, Serialize)]
pub struct WateringJob {
    pub plant_index: usize,
    pub duration_ms: usize,
}
