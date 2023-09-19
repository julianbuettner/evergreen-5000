use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LastSeenResponse {
    pub last_seen_timestamp: i64,
    pub last_battery_percentage: f32,
    pub last_watering_date: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WateringJob {
    pub plant_index: usize,
    pub duration_ms: usize,
}
