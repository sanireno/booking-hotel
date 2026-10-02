use crate::users::model::{CreateUser, UpdateUser, User};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_user(pool: &PgPool, user: CreateUser) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, password_hash, first_name, last_name, role)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, email, password_hash, first_name, last_name, role, created_at
        "#,
    )
    .bind(user.email)
    .bind(user.password_hash)
    .bind(user.first_name)
    .bind(user.last_name)
    .bind(user.role)
    .fetch_one(pool)
    .await
}

pub async fn get_user(pool: &PgPool, id: Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, first_name, last_name, role, created_at
        FROM users WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, first_name, last_name, role, created_at
        FROM users WHERE LOWER(email) = LOWER($1)
        "#,
    )
    .bind(email)
    .fetch_one(pool)
    .await
}

pub async fn update_user(pool: &PgPool, id: Uuid, user: UpdateUser) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET email = $1, password_hash = $2, first_name = $3, last_name = $4, role = $5
        WHERE id = $6
        RETURNING id, email, password_hash, first_name, last_name, role, created_at
        "#,
    )
    .bind(user.email)
    .bind(user.password_hash)
    .bind(user.first_name)
    .bind(user.last_name)
    .bind(user.role)
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        DELETE FROM users WHERE id = $1
        RETURNING id, email, password_hash, first_name, last_name, role, created_at
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn get_all_users(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, email, password_hash, first_name, last_name, role, created_at
        FROM users ORDER BY id LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}
