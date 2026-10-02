use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("Request body is too large")]
    PayloadTooLarge,
    #[error("Expected application/json")]
    UnsupportedMediaType,
    #[error("Internal server error")]
    InternalServerError,
    #[error("Not found")]
    NotFound,
    #[error("Conflict")]
    Conflict,
    #[error("{0}")]
    ConflictMessage(String),
    #[error("{0}")]
    Validation(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::RowNotFound => AppError::NotFound,
            sqlx::Error::Database(database_error) => match database_error.code().as_deref() {
                Some("23505") => AppError::Conflict, //23505 — нарушение уникальности
                Some("23503") => {
                    //23503 — нарушение внешнего ключа
                    AppError::Validation("Referenced resource does not exist".to_string())
                }
                Some("23514") => {
                    //23514 — нарушение CHECK
                    AppError::Validation("Data violates database constraints".to_string())
                }
                Some("23502") => AppError::Validation("Required field is missing".to_string()), //23502 — нарушение NOT NULL
                _ => {
                    tracing::error!(%database_error, "database error");
                    AppError::InternalServerError
                }
            },
            error => {
                eprintln!("DATABASE ERROR: {error}");
                AppError::InternalServerError
            }
        }
    }
}

impl From<axum::extract::rejection::JsonRejection> for AppError {
    fn from(error: axum::extract::rejection::JsonRejection) -> Self {
        match error.status() {
            StatusCode::PAYLOAD_TOO_LARGE => AppError::PayloadTooLarge,
            StatusCode::UNSUPPORTED_MEDIA_TYPE => AppError::UnsupportedMediaType,
            StatusCode::UNPROCESSABLE_ENTITY => {
                AppError::Validation("Invalid request fields".into())
            }
            _ => AppError::BadRequest("Expected a valid JSON body".into()),
        }
    }
}

impl From<axum::extract::rejection::PathRejection> for AppError {
    fn from(_: axum::extract::rejection::PathRejection) -> Self {
        AppError::BadRequest("Invalid resource identifier".into())
    }
}

impl From<axum::extract::rejection::QueryRejection> for AppError {
    fn from(_: axum::extract::rejection::QueryRejection) -> Self {
        AppError::BadRequest("Invalid query parameters".into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "Request body is too large".into(),
            ),
            AppError::UnsupportedMediaType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Expected application/json".into(),
            ),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Conflict => (StatusCode::CONFLICT, "Conflict".to_string()),
            AppError::ConflictMessage(message) => (StatusCode::CONFLICT, message),
            AppError::Validation(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
        };
        let body = Json(json!({
            "error": message
        }));
        let mut response = (status, body).into_response();
        if status == StatusCode::UNAUTHORIZED {
            response.headers_mut().insert(
                axum::http::header::WWW_AUTHENTICATE,
                axum::http::HeaderValue::from_static("Bearer"),
            );
        }
        response
    }
}
