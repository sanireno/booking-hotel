use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PricePlan {
    pub id: Uuid,

    pub room_type_id: Uuid,

    pub name: String,

    pub price_per_night: Decimal,

    pub refundable: bool,
}
