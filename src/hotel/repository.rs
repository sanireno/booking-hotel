use crate::hotel::model::{CreateHotel, Hotel, UpdateHotel};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_hotel(pool: &PgPool, hotel:CreateHotel)->Result<Hotel,sqlx::Error>{
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
pub async fn delete_hotel(pool:&PgPool,id:Uuid)->Result<Hotel,sqlx::Error>{
    sqlx::query_as::<_,Hotel>(
        r#"
            DELETE FROM hotels
            WHERE id=$1
            RETURNING id,name,description,city,address,created_at
            "#,
    )
        .bind(id)
        .fetch_one(pool)
        .await
}
pub async fn update_hotel(pool:&PgPool,id:Uuid,hotel:UpdateHotel)->Result<Hotel,sqlx::Error>{
    sqlx::query_as::<_,Hotel>(
        r#"
            UPDATE hotels
            SET
                name=$1,
                description=$2,
                city=$3,
                address=$4
            WHERE id=$5
            RETURNING id,name,description,city,address,created_at
            "#,
    )
        .bind(hotel.name)
        .bind(hotel.description)
        .bind(hotel.city)
        .bind(hotel.address)
        .bind(id)
        .fetch_one(pool)
        .await
}
pub async fn get_all_hotels(pool:&PgPool,limit:i64,offset:i64)->Result<Vec<Hotel>,sqlx::Error>{
    sqlx::query_as::<_,Hotel>(
        r#"
            SELECT id,name,description,city,address,created_at
            FROM hotels
            ORDER BY id
            LIMIT $1 OFFSET $2
            "#,
    )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}