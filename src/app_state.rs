use crate::{
    app_error::AppError,
    auth::{password::PasswordService, token::TokenService},
    config::AuthConfig,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub tokens: TokenService,
    pub passwords: PasswordService,
}

impl AppState {
    pub async fn new(pool: PgPool, config: &AuthConfig) -> Result<Self, AppError> {
        let tokens = TokenService::new(config).map_err(|error| {
            tracing::error!(%error, "invalid authentication configuration");
            AppError::InternalServerError
        })?;
        Ok(Self {
            pool,
            tokens,
            passwords: PasswordService::new().await?,
        })
    }
}
