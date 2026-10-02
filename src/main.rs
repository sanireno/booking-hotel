pub mod app_error;
pub mod app_state;
pub mod auth;
pub mod bookings;
pub mod config;
pub mod hotel;
pub mod pricing;
pub mod rooms;
pub mod users;

#[cfg(test)]
mod repository_tests;

use app_state::AppState;
use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use config::Config;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Config::from_env()?;
    let address = config.server_address()?;
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;
    info!("database migrations applied");

    let state = AppState::new(pool, &config.auth).await?;

    let listener = TcpListener::bind(address).await?;
    info!(%address, "server started");

    axum::serve(listener, app(state)).await?;

    Ok(())
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .nest("/auth", auth::routes())
        .with_state(state)
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    database: &'static str,
}

async fn health(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                database: "up",
            }),
        ),
        Err(error) => {
            error!(%error, "database health check failed");

            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "degraded",
                    database: "down",
                }),
            )
        }
    }
}
