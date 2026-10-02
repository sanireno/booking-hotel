use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,

    pub email: String,
    pub password_hash: String,

    pub first_name: String,
    pub last_name: String,

    pub role: UserRole,

    pub created_at: DateTime<Utc>,
}

/// Internal persistence input: the service supplies a hash and authorizes the role.
#[derive(Debug)]
pub struct CreateUser {
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
}

#[derive(Debug)]
pub struct UpdateUser {
    pub email: String,
    pub password_hash: String,
    pub first_name: String,
    pub last_name: String,
    pub role: UserRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum UserRole {
    Customer,
    Manager,
    Admin,
}
