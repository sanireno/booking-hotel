use crate::{
    app_error::AppError,
    app_state::AppState,
    auth::extractor::AuthUser,
    hotel::{
        dto::{CreateHotelRequest, HotelListQuery, HotelResponse, UpdateHotelRequest},
        service,
    },
};
use axum::{
    Json,
    extract::{
        Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::{StatusCode, header::LOCATION},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

pub async fn create_hotel(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
    payload: Result<Json<CreateHotelRequest>, JsonRejection>,
) -> Result<Response, AppError> {
    let Json(request) = payload?;
    let hotel = service::create_hotel(&state.pool, &actor, request).await?;
    let location = format!("/hotels/{}", hotel.id);
    Ok((StatusCode::CREATED, [(LOCATION, location)], Json(hotel)).into_response())
}

pub async fn get_hotel(
    State(state): State<AppState>,
    path: Result<Path<Uuid>, PathRejection>,
) -> Result<Json<HotelResponse>, AppError> {
    let Path(id) = path?;
    Ok(Json(service::get_hotel(&state.pool, id).await?))
}

pub async fn get_all_hotels(
    State(state): State<AppState>,
    query: Result<Query<HotelListQuery>, QueryRejection>,
) -> Result<Json<Vec<HotelResponse>>, AppError> {
    let Query(query) = query?;
    Ok(Json(service::get_all_hotels(&state.pool, query).await?))
}

pub async fn update_hotel(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
    path: Result<Path<Uuid>, PathRejection>,
    payload: Result<Json<UpdateHotelRequest>, JsonRejection>,
) -> Result<Json<HotelResponse>, AppError> {
    let Path(id) = path?;
    let Json(request) = payload?;
    Ok(Json(
        service::update_hotel(&state.pool, &actor, id, request).await?,
    ))
}

pub async fn delete_hotel(
    State(state): State<AppState>,
    AuthUser(actor): AuthUser,
    path: Result<Path<Uuid>, PathRejection>,
) -> Result<StatusCode, AppError> {
    let Path(id) = path?;
    service::delete_hotel(&state.pool, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
