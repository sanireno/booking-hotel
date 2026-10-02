use std::{env, net::SocketAddr};

use thiserror::Error;

const DEFAULT_APP_HOST: &str = "127.0.0.1";
const DEFAULT_APP_PORT: u16 = 3000;
const DEFAULT_DATABASE_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_JWT_TTL_SECONDS: u64 = 3600;

#[derive(Clone)]
pub struct Config {
    pub app_host: String,
    pub app_port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub auth: AuthConfig,
}

#[derive(Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_ttl_seconds: u64,
}

impl AuthConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.jwt_secret.len() < 32 || self.jwt_secret.trim().is_empty() {
            return Err(ConfigError::InvalidVariable(
                "JWT_SECRET (at least 32 bytes)",
            ));
        }
        if !(1..=86400).contains(&self.jwt_ttl_seconds) {
            return Err(ConfigError::InvalidVariable("JWT_TTL_SECONDS (1..=86400)"));
        }
        Ok(())
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let auth = AuthConfig {
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| ConfigError::MissingVariable("JWT_SECRET"))?,
            jwt_ttl_seconds: parse_optional("JWT_TTL_SECONDS", DEFAULT_JWT_TTL_SECONDS)?,
        };
        auth.validate()?;
        Ok(Self {
            app_host: env::var("APP_HOST").unwrap_or_else(|_| DEFAULT_APP_HOST.to_owned()),
            app_port: parse_optional("APP_PORT", DEFAULT_APP_PORT)?,
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingVariable("DATABASE_URL"))?,
            database_max_connections: parse_optional(
                "DATABASE_MAX_CONNECTIONS",
                DEFAULT_DATABASE_MAX_CONNECTIONS,
            )?,
            auth,
        })
    }

    pub fn server_address(&self) -> Result<SocketAddr, ConfigError> {
        let address = format!("{}:{}", self.app_host, self.app_port);

        address
            .parse()
            .map_err(|_| ConfigError::InvalidServerAddress(address))
    }
}

fn parse_optional<T>(name: &'static str, default: T) -> Result<T, ConfigError>
where
    T: std::str::FromStr,
{
    match env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| ConfigError::InvalidVariable(name)),
        Err(_) => Ok(default),
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("required environment variable {0} is missing")]
    MissingVariable(&'static str),
    #[error("environment variable {0} has an invalid value")]
    InvalidVariable(&'static str),
    #[error("APP_HOST and APP_PORT form an invalid socket address: {0}")]
    InvalidServerAddress(String),
}
