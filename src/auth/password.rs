use crate::app_error::AppError;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use std::sync::Arc;
use tokio::{sync::Semaphore, task::spawn_blocking};

#[derive(Clone)]
pub struct PasswordService {
    dummy_hash: Arc<str>,
    slots: Arc<Semaphore>,
}

impl PasswordService {
    pub async fn new() -> Result<Self, AppError> {
        let dummy_hash = spawn_blocking(|| hash_password("dummy-account-password"))
            .await
            .map_err(blocking_error)??;
        Ok(Self {
            dummy_hash: dummy_hash.into(),
            slots: Arc::new(Semaphore::new(2)),
        })
    }

    pub async fn hash(&self, password: String) -> Result<String, AppError> {
        let permit = self
            .slots
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        spawn_blocking(move || {
            let _permit = permit;
            hash_password(&password)
        })
        .await
        .map_err(blocking_error)?
    }

    pub async fn verify(&self, password: String, hash: Option<String>) -> Result<bool, AppError> {
        let hash = hash
            .map(Arc::<str>::from)
            .unwrap_or_else(|| self.dummy_hash.clone());
        let permit = self
            .slots
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        spawn_blocking(move || {
            let _permit = permit;
            let parsed_hash = PasswordHash::new(&hash).map_err(password_error)?;
            match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
                Ok(()) => Ok(true),
                Err(argon2::password_hash::Error::Password) => Ok(false),
                Err(error) => Err(password_error(error)),
            }
        })
        .await
        .map_err(blocking_error)?
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(password_error)
}

fn password_error(error: argon2::password_hash::Error) -> AppError {
    tracing::error!(%error, "password processing failed");
    AppError::InternalServerError
}

fn blocking_error(error: tokio::task::JoinError) -> AppError {
    tracing::error!(%error, "password worker failed");
    AppError::InternalServerError
}
