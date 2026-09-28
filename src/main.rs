pub mod app_error;
pub mod auth;
pub mod bookings;
pub mod config;
pub mod hotel;
pub mod pricing;
pub mod rooms;
pub mod users;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use config::Config;
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
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

    let listener = TcpListener::bind(address).await?;
    info!(%address, "server started");

    axum::serve(listener, app(pool)).await?;

    Ok(())
}

fn app(pool: PgPool) -> Router {
    Router::new().route("/health", get(health)).with_state(pool)
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

async fn health(State(pool): State<PgPool>) -> (StatusCode, Json<HealthResponse>) {
    match sqlx::query("SELECT 1").execute(&pool).await {
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
