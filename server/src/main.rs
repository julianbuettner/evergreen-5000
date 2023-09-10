use std::net::SocketAddr;

use api::last_seen;
use axum::{
    routing::{get, post},
    Router,
};

use config::ConfigManager;
use state::JsonStateManager;

use crate::{
    api::{get_plant, test_watering},
    watering_test::PendingWateringTest,
};

mod api;
mod config;
mod model;
mod state;
mod watering_test;

#[derive(Clone)]
pub struct GlobalState {
    pub config: ConfigManager,
    pub json_state: JsonStateManager,
    pub pending_warting_test: PendingWateringTest,
}

#[tokio::main]
async fn main() {
    let statemanager = JsonStateManager::new();
    if let Err(err) = statemanager.ensure_state() {
        eprintln!("Something is wrong with the state file.\n{}", err);
        eprintln!("Try fixing or deleting state.json and restart the program.");
        return;
    }
    let configmanager = ConfigManager::new();
    let plants_result = configmanager.get_plant_config();
    if let Err(err) = plants_result {
        eprintln!("Invalid config.\n{}", err);
        eprintln!("Please fix the configuration and restart the program.");
        return;
    } else {
        println!("{} plants are configured.", plants_result.unwrap().len());
    }

    let host = configmanager.get_host().unwrap();
    let port = configmanager.get_port().unwrap();
    println!("Listening on {}:{}", host, port);

    let state = GlobalState {
        config: configmanager,
        json_state: statemanager,
        pending_warting_test: PendingWateringTest::new(),
    };
    let app = Router::new()
        .route("/lastseen", get(last_seen))
        .route("/plants", get(get_plant))
        .route("/testwatering/:plantname", post(test_watering))
        .with_state(state);

    let addr = SocketAddr::from((host, port));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
