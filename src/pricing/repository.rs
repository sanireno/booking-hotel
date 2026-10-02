use crate::pricing::model::{CreatePricePlan, PricePlan, UpdatePricePlan};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_price_plan(
    pool: &PgPool,
    plan: CreatePricePlan,
) -> Result<PricePlan, sqlx::Error> {
    sqlx::query_as::<_, PricePlan>(
        r#"
        INSERT INTO price_plans (room_type_id, name, price_per_night, refundable)
        VALUES ($1, $2, $3, $4)
        RETURNING id, room_type_id, name, price_per_night, refundable
        "#,
    )
    .bind(plan.room_type_id)
    .bind(plan.name)
    .bind(plan.price_per_night)
    .bind(plan.refundable)
    .fetch_one(pool)
    .await
}

pub async fn get_price_plan(pool: &PgPool, id: Uuid) -> Result<PricePlan, sqlx::Error> {
    sqlx::query_as::<_, PricePlan>(
        r#"
        SELECT id, room_type_id, name, price_per_night, refundable
        FROM price_plans WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn update_price_plan(
    pool: &PgPool,
    id: Uuid,
    plan: UpdatePricePlan,
) -> Result<PricePlan, sqlx::Error> {
    sqlx::query_as::<_, PricePlan>(
        r#"
        UPDATE price_plans
        SET room_type_id = $1, name = $2, price_per_night = $3, refundable = $4
        WHERE id = $5
        RETURNING id, room_type_id, name, price_per_night, refundable
        "#,
    )
    .bind(plan.room_type_id)
    .bind(plan.name)
    .bind(plan.price_per_night)
    .bind(plan.refundable)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_price_plan(pool: &PgPool, id: Uuid) -> Result<PricePlan, sqlx::Error> {
    sqlx::query_as::<_, PricePlan>(
        r#"
        DELETE FROM price_plans WHERE id = $1
        RETURNING id, room_type_id, name, price_per_night, refundable
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn get_all_price_plans(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<PricePlan>, sqlx::Error> {
    sqlx::query_as::<_, PricePlan>(
        r#"
        SELECT id, room_type_id, name, price_per_night, refundable
        FROM price_plans ORDER BY id LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_price_plans_by_room_type(
    pool: &PgPool,
    room_type_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<PricePlan>, sqlx::Error> {
    sqlx::query_as::<_, PricePlan>(
        r#"
        SELECT id, room_type_id, name, price_per_night, refundable
        FROM price_plans WHERE room_type_id = $1 ORDER BY id LIMIT $2 OFFSET $3
        "#,
    )
    .bind(room_type_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
