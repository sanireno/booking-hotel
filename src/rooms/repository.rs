use crate::rooms::model::{CreateRoom, CreateRoomType, Room, RoomType, UpdateRoom, UpdateRoomType};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_room_type(
    pool: &PgPool,
    room_type: CreateRoomType,
) -> Result<RoomType, sqlx::Error> {
    sqlx::query_as::<_, RoomType>(
        r#"
        INSERT INTO room_types (hotel_id, name, description, capacity, base_price)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, hotel_id, name, description, capacity, base_price
        "#,
    )
    .bind(room_type.hotel_id)
    .bind(room_type.name)
    .bind(room_type.description)
    .bind(room_type.capacity)
    .bind(room_type.base_price)
    .fetch_one(pool)
    .await
}

pub async fn get_room_type(pool: &PgPool, id: Uuid) -> Result<RoomType, sqlx::Error> {
    sqlx::query_as::<_, RoomType>(
        r#"
        SELECT id, hotel_id, name, description, capacity, base_price
        FROM room_types WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn update_room_type(
    pool: &PgPool,
    id: Uuid,
    room_type: UpdateRoomType,
) -> Result<RoomType, sqlx::Error> {
    sqlx::query_as::<_, RoomType>(
        r#"
        UPDATE room_types
        SET hotel_id = $1, name = $2, description = $3, capacity = $4, base_price = $5
        WHERE id = $6
        RETURNING id, hotel_id, name, description, capacity, base_price
        "#,
    )
    .bind(room_type.hotel_id)
    .bind(room_type.name)
    .bind(room_type.description)
    .bind(room_type.capacity)
    .bind(room_type.base_price)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_room_type(pool: &PgPool, id: Uuid) -> Result<RoomType, sqlx::Error> {
    sqlx::query_as::<_, RoomType>(
        r#"
        DELETE FROM room_types WHERE id = $1
        RETURNING id, hotel_id, name, description, capacity, base_price
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn get_all_room_types(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<RoomType>, sqlx::Error> {
    sqlx::query_as::<_, RoomType>(
        r#"
        SELECT id, hotel_id, name, description, capacity, base_price
        FROM room_types ORDER BY id LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_room_types_by_hotel(
    pool: &PgPool,
    hotel_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<RoomType>, sqlx::Error> {
    sqlx::query_as::<_, RoomType>(
        r#"
        SELECT id, hotel_id, name, description, capacity, base_price
        FROM room_types WHERE hotel_id = $1 ORDER BY id LIMIT $2 OFFSET $3
        "#,
    )
    .bind(hotel_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn create_room(pool: &PgPool, room: CreateRoom) -> Result<Room, sqlx::Error> {
    sqlx::query_as::<_, Room>(
        r#"
        INSERT INTO rooms (room_type_id, number) VALUES ($1, $2)
        RETURNING id, room_type_id, number
        "#,
    )
    .bind(room.room_type_id)
    .bind(room.number)
    .fetch_one(pool)
    .await
}

pub async fn get_room(pool: &PgPool, id: Uuid) -> Result<Room, sqlx::Error> {
    sqlx::query_as::<_, Room>("SELECT id, room_type_id, number FROM rooms WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn update_room(pool: &PgPool, id: Uuid, room: UpdateRoom) -> Result<Room, sqlx::Error> {
    sqlx::query_as::<_, Room>(
        r#"
        UPDATE rooms SET room_type_id = $1, number = $2 WHERE id = $3
        RETURNING id, room_type_id, number
        "#,
    )
    .bind(room.room_type_id)
    .bind(room.number)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_room(pool: &PgPool, id: Uuid) -> Result<Room, sqlx::Error> {
    sqlx::query_as::<_, Room>("DELETE FROM rooms WHERE id = $1 RETURNING id, room_type_id, number")
        .bind(id)
        .fetch_one(pool)
        .await
}

pub async fn get_all_rooms(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Room>, sqlx::Error> {
    sqlx::query_as::<_, Room>(
        "SELECT id, room_type_id, number FROM rooms ORDER BY id LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_rooms_by_room_type(
    pool: &PgPool,
    room_type_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Room>, sqlx::Error> {
    sqlx::query_as::<_, Room>(
        r#"
        SELECT id, room_type_id, number
        FROM rooms WHERE room_type_id = $1 ORDER BY id LIMIT $2 OFFSET $3
        "#,
    )
    .bind(room_type_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_rooms_by_hotel(
    pool: &PgPool,
    hotel_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Room>, sqlx::Error> {
    sqlx::query_as::<_, Room>(
        r#"
        SELECT r.id, r.room_type_id, r.number
        FROM rooms r JOIN room_types rt ON rt.id = r.room_type_id
        WHERE rt.hotel_id = $1 ORDER BY r.id LIMIT $2 OFFSET $3
        "#,
    )
    .bind(hotel_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
