pub mod dto;
pub mod handler;
pub mod model;
pub mod repository;
pub mod service;

use crate::app_state::AppState;
use axum::{Router, extract::DefaultBodyLimit, routing::get};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/hotels",
            get(handler::get_all_hotels).post(handler::create_hotel),
        )
        .route(
            "/hotels/{id}",
            get(handler::get_hotel)
                .put(handler::update_hotel)
                .delete(handler::delete_hotel),
        )
        .layer(DefaultBodyLimit::max(32 * 1024))
}

#[cfg(test)]
mod tests;
