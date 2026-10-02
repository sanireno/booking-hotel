use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Booking {
    pub id: Uuid,

    pub user_id: Uuid,
    pub room_id: Uuid,
    pub price_plan_id: Uuid,
    pub check_in: NaiveDate,
    pub check_out: NaiveDate,

    pub guests: i32,

    pub status: Status,

    pub total_price: Decimal,

    pub created_at: DateTime<Utc>,
}

/// The service validates availability, capacity, price plan and total price.
#[derive(Debug)]
pub struct CreateBooking {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub price_plan_id: Uuid,
    pub check_in: NaiveDate,
    pub check_out: NaiveDate,
    pub guests: i32,
    pub total_price: Decimal,
}

#[derive(Debug)]
pub struct UpdateBooking {
    pub user_id: Uuid,
    pub room_id: Uuid,
    pub price_plan_id: Uuid,
    pub check_in: NaiveDate,
    pub check_out: NaiveDate,
    pub guests: i32,
    pub status: Status,
    pub total_price: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Status {
    Pending,
    Confirmed,
    Cancelled,
}
