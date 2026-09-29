use sqlx::PgPool;
use uuid::Uuid;
use crate::app_error::AppError;
use crate::hotel::model::{CreateHotel, Hotel};

pub async fn create_hotel(pool:&PgPool, hotel:CreateHotel)->Result<Hotel,sqlx::Error>{
    sqlx::query_as::<_, Hotel>(
        r#"
            INSERT INTO hotels(name,description,city,address)
            VALUES ($1,$2,$3,$4)
            RETURNING id,name,description,city,address,created_at
            "#,
    )
        .bind(hotel.name)
        .bind(hotel.description)
        .bind(hotel.city)
        .bind(hotel.address)
        .fetch_one(pool)
        .await
}
pub async fn get_hotel(pool:&PgPool,id:Uuid)->Result<Hotel,sqlx::Error>{
    sqlx::query_as::<_,Hotel>(
        r#"
            SELECT id,name,description,city,address,created_at
            FROM hotels
            WHERE id=$1
            "#,
    )
        .bind(id)
        .fetch_one(pool)
        .await
}