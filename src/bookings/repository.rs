//! Persistence only. The service must validate booking rules before calling mutations.

use crate::bookings::model::{Booking, CreateBooking, Status, UpdateBooking};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_booking(pool: &PgPool, booking: CreateBooking) -> Result<Booking, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        r#"
        INSERT INTO bookings (user_id, room_id, price_plan_id, check_in, check_out, guests, total_price)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, user_id, room_id, price_plan_id, check_in, check_out,
                  guests, status, total_price, created_at
        "#,
    )
    .bind(booking.user_id)
    .bind(booking.room_id)
    .bind(booking.price_plan_id)
    .bind(booking.check_in)
    .bind(booking.check_out)
    .bind(booking.guests)
    .bind(booking.total_price)
    .fetch_one(pool)
    .await
}

pub async fn get_booking(pool: &PgPool, id: Uuid) -> Result<Booking, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        r#"
        SELECT id, user_id, room_id, price_plan_id, check_in, check_out,
               guests, status, total_price, created_at
        FROM bookings WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn update_booking(
    pool: &PgPool,
    id: Uuid,
    booking: UpdateBooking,
) -> Result<Booking, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        r#"
        UPDATE bookings
        SET user_id = $1, room_id = $2, price_plan_id = $3, check_in = $4,
            check_out = $5, guests = $6, status = $7, total_price = $8
        WHERE id = $9
        RETURNING id, user_id, room_id, price_plan_id, check_in, check_out,
                  guests, status, total_price, created_at
        "#,
    )
    .bind(booking.user_id)
    .bind(booking.room_id)
    .bind(booking.price_plan_id)
    .bind(booking.check_in)
    .bind(booking.check_out)
    .bind(booking.guests)
    .bind(booking.status)
    .bind(booking.total_price)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn update_booking_status(
    pool: &PgPool,
    id: Uuid,
    status: Status,
) -> Result<Booking, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        r#"
        UPDATE bookings SET status = $1 WHERE id = $2
        RETURNING id, user_id, room_id, price_plan_id, check_in, check_out,
                  guests, status, total_price, created_at
        "#,
    )
    .bind(status)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_booking(pool: &PgPool, id: Uuid) -> Result<Booking, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        r#"
        DELETE FROM bookings WHERE id = $1
        RETURNING id, user_id, room_id, price_plan_id, check_in, check_out,
                  guests, status, total_price, created_at
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn get_all_bookings(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Booking>, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        r#"
        SELECT id, user_id, room_id, price_plan_id, check_in, check_out,
               guests, status, total_price, created_at
        FROM bookings ORDER BY id LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_bookings_by_user(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Booking>, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        r#"
        SELECT id, user_id, room_id, price_plan_id, check_in, check_out,
               guests, status, total_price, created_at
        FROM bookings WHERE user_id = $1 ORDER BY id LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_bookings_by_room(
    pool: &PgPool,
    room_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Booking>, sqlx::Error> {
    sqlx::query_as::<_, Booking>(
        r#"
        SELECT id, user_id, room_id, price_plan_id, check_in, check_out,
               guests, status, total_price, created_at
        FROM bookings WHERE room_id = $1 ORDER BY id LIMIT $2 OFFSET $3
        "#,
    )
    .bind(room_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
