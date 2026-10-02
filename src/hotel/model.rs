use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Hotel {
    pub id: Uuid,

    pub name: String,
    pub description: Option<String>,

    pub city: String,
    pub address: String,

    pub created_at: DateTime<Utc>,
}
#[derive(Debug)]
pub struct CreateHotel {
    pub name: String,
    pub description: Option<String>,
    pub city: String,
    pub address: String,
}

#[derive(Debug)]
pub struct UpdateHotel {
    pub name: String,
    pub description: Option<String>,
    pub city: String,
    pub address: String,
}
