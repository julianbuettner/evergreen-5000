use std::net::SocketAddr;

use axum::Router;
use chrono::NaiveDate;
use stateholder::{State, Stateholder};
use tokio::net::TcpListener;

mod api;
mod model;
mod stateholder;

#[tokio::main]
async fn main() {
    let stateholder = Stateholder::new();
    let app = Router::new().with_state(stateholder);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
