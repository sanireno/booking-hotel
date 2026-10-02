pub mod dto;
pub mod extractor;
pub mod handler;
pub mod model;
pub mod password;
pub mod service;
pub mod token;

use crate::app_state::AppState;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(handler::register))
        .route("/login", post(handler::login))
        .route("/me", get(handler::me))
        .layer(DefaultBodyLimit::max(16 * 1024))
}

#[cfg(test)]
mod tests;
