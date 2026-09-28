use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RoomType {
    pub id: Uuid,
    pub hotel_id: Uuid,

    pub name: String,
    pub description: Option<String>,

    pub capacity: i32,

    pub base_price: Decimal,
}
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Room {
    pub id: Uuid,

    pub room_type_id: Uuid,

    pub number: String,
}
