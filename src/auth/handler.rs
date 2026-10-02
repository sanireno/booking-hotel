use crate::{
    app_error::AppError,
    app_state::AppState,
    auth::{
        dto::{LoginRequest, RegisterRequest},
        extractor::AuthUser,
        service,
    },
};
use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    http::{
        StatusCode,
        header::{CACHE_CONTROL, PRAGMA},
    },
    response::{IntoResponse, Response},
};

pub async fn register(
    State(state): State<AppState>,
    payload: Result<Json<RegisterRequest>, JsonRejection>,
) -> Result<Response, AppError> {
    let Json(request) = payload.map_err(json_error)?;
    let response = service::register(&state, request).await?;
    Ok((
        StatusCode::CREATED,
        [(CACHE_CONTROL, "no-store"), (PRAGMA, "no-cache")],
        Json(response),
    )
        .into_response())
}

pub async fn login(
    State(state): State<AppState>,
    payload: Result<Json<LoginRequest>, JsonRejection>,
) -> Result<Response, AppError> {
    let Json(request) = payload.map_err(json_error)?;
    let response = service::login(&state, request).await?;
    Ok((
        [(CACHE_CONTROL, "no-store"), (PRAGMA, "no-cache")],
        Json(response),
    )
        .into_response())
}

pub async fn me(AuthUser(user): AuthUser) -> Response {
    ([(CACHE_CONTROL, "no-store")], Json(user)).into_response()
}

fn json_error(error: JsonRejection) -> AppError {
    match error.status() {
        StatusCode::PAYLOAD_TOO_LARGE => AppError::PayloadTooLarge,
        StatusCode::UNSUPPORTED_MEDIA_TYPE => AppError::UnsupportedMediaType,
        StatusCode::UNPROCESSABLE_ENTITY => AppError::Validation("Invalid request fields".into()),
        _ => AppError::BadRequest("Expected a valid JSON body".into()),
    }
}
