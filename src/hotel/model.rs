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
