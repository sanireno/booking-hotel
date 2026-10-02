use crate::{
    app_error::AppError,
    auth::dto::UserResponse,
    hotel::{
        dto::{CreateHotelRequest, HotelListQuery, HotelResponse, UpdateHotelRequest},
        model::{CreateHotel, UpdateHotel},
        repository,
    },
    users::model::UserRole,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_hotel(
    pool: &PgPool,
    actor: &UserResponse,
    request: CreateHotelRequest,
) -> Result<HotelResponse, AppError> {
    authorize(actor)?;
    let hotel = repository::create_hotel(
        pool,
        CreateHotel {
            name: required_text(&request.name, "name", 200)?,
            description: description(request.description)?,
            city: required_text(&request.city, "city", 100)?,
            address: required_text(&request.address, "address", 500)?,
        },
    )
    .await?;
    Ok(hotel.into())
}

pub async fn get_hotel(pool: &PgPool, id: Uuid) -> Result<HotelResponse, AppError> {
    Ok(repository::get_hotel(pool, id).await?.into())
}

pub async fn get_all_hotels(
    pool: &PgPool,
    query: HotelListQuery,
) -> Result<Vec<HotelResponse>, AppError> {
    if !(1..=100).contains(&query.limit) || query.offset < 0 {
        return Err(AppError::Validation(
            "limit must be 1 to 100 and offset must be non-negative".into(),
        ));
    }
    Ok(repository::get_all_hotels(pool, query.limit, query.offset)
        .await?
        .into_iter()
        .map(HotelResponse::from)
        .collect())
}

pub async fn update_hotel(
    pool: &PgPool,
    actor: &UserResponse,
    id: Uuid,
    request: UpdateHotelRequest,
) -> Result<HotelResponse, AppError> {
    authorize(actor)?;
    let hotel = repository::update_hotel(
        pool,
        id,
        UpdateHotel {
            name: required_text(&request.name, "name", 200)?,
            description: description(request.description)?,
            city: required_text(&request.city, "city", 100)?,
            address: required_text(&request.address, "address", 500)?,
        },
    )
    .await?;
    Ok(hotel.into())
}

pub async fn delete_hotel(pool: &PgPool, actor: &UserResponse, id: Uuid) -> Result<(), AppError> {
    authorize(actor)?;
    match repository::delete_hotel(pool, id).await {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(error)) if error.code().as_deref() == Some("23503") => Err(
            AppError::ConflictMessage("Hotel has bookings and cannot be deleted".into()),
        ),
        Err(error) => Err(error.into()),
    }
}

fn authorize(actor: &UserResponse) -> Result<(), AppError> {
    if actor.role == UserRole::Admin {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn required_text(value: &str, field: &str, max_length: usize) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max_length || value.chars().any(char::is_control)
    {
        return Err(AppError::Validation(format!(
            "{field} must contain 1 to {max_length} characters without control characters"
        )));
    }
    Ok(value.into())
}

fn description(value: Option<String>) -> Result<Option<String>, AppError> {
    let Some(value) = value else { return Ok(None) };
    let value = value.trim();
    if value.chars().count() > 5000
        || value
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err(AppError::Validation("description must contain at most 5000 characters without unsupported control characters".into()));
    }
    Ok((!value.is_empty()).then(|| value.into()))
}
