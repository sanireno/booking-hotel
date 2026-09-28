# Booking Hotel API

Backend service for a hotel booking system written in Rust with Axum, SQLx, and PostgreSQL.

## Local setup

Requirements: Rust and PostgreSQL 13+ (or Docker with Docker Compose).

```bash
cp .env.example .env
docker compose up -d postgres
cargo run
```

The application connects to PostgreSQL and applies all SQLx migrations automatically on startup.

Check the application and database:

```bash
curl http://127.0.0.1:3000/health
```

Expected response:

```json
{"status":"ok","database":"up"}
```

## Database schema

The initial migration creates the following tables:

- `users`
- `hotels`
- `room_types`
- `rooms`
- `price_plans`
- `bookings`

The schema contains foreign keys, domain constraints, case-insensitive unique user emails, and indexes for hotel search and booking queries.

To apply migrations manually with SQLx CLI:

```bash
cargo sqlx migrate run
```
