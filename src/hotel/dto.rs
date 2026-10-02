use crate::hotel::model::Hotel;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateHotelRequest {
    pub name: String,
    pub description: Option<String>,
    pub city: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateHotelRequest {
    pub name: String,
    pub description: Option<String>,
    pub city: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HotelListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct HotelResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub city: String,
    pub address: String,
    pub created_at: DateTime<Utc>,
}

impl From<Hotel> for HotelResponse {
    fn from(hotel: Hotel) -> Self {
        Self {
            id: hotel.id,
            name: hotel.name,
            description: hotel.description,
            city: hotel.city,
            address: hotel.address,
            created_at: hotel.created_at,
        }
    }
}
