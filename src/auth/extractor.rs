use crate::{
    app_error::AppError,
    app_state::AppState,
    auth::{dto::UserResponse, service},
    users::model::UserRole,
};
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

#[derive(Debug)]
pub struct AuthUser(pub UserResponse);

impl AuthUser {
    pub fn require_role(&self, role: UserRole) -> Result<(), AppError> {
        if self.0.role == role {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut headers = parts.headers.get_all(AUTHORIZATION).iter();
        let header = headers.next().ok_or(AppError::Unauthorized)?;
        if headers.next().is_some() {
            return Err(AppError::Unauthorized);
        }
        let value = header.to_str().map_err(|_| AppError::Unauthorized)?;
        let mut fields = value.split_ascii_whitespace();
        let scheme = fields.next().ok_or(AppError::Unauthorized)?;
        let token = fields.next().ok_or(AppError::Unauthorized)?;
        if !scheme.eq_ignore_ascii_case("Bearer") || fields.next().is_some() {
            return Err(AppError::Unauthorized);
        }
        Ok(Self(service::authenticate(state, token).await?))
    }
}
