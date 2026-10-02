use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PricePlan {
    pub id: Uuid,

    pub room_type_id: Uuid,

    pub name: String,

    pub price_per_night: Decimal,

    pub refundable: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreatePricePlan {
    pub room_type_id: Uuid,
    pub name: String,
    pub price_per_night: Decimal,
    pub refundable: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePricePlan {
    pub room_type_id: Uuid,
    pub name: String,
    pub price_per_night: Decimal,
    pub refundable: bool,
}
