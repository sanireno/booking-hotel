use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Hotel {
    pub id: Uuid,

    pub name: String,
    pub description: Option<String>,

    pub city: String,
    pub address: String,

    pub created_at: DateTime<Utc>,
}
#[derive(Debug, Deserialize)]
pub struct CreateHotel {
    pub name: String,
    pub description: Option<String>,
    pub city: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHotel {
    pub name: String,
    pub description: Option<String>,
    pub city: String,
    pub address: String,
}
