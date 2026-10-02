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

## Repositories

Repositories use the existing function-based SQLx API: `&PgPool` inputs and
`Result<Model, sqlx::Error>` outputs. Single-record reads, updates and deletes
return `sqlx::Error::RowNotFound` when the ID does not exist. Mutations return
the persisted row via `RETURNING`; IDs and creation timestamps come from PostgreSQL.

| Module | Records | Additional reads |
| --- | --- | --- |
| `hotel` | Hotels | All hotels |
| `users` | Users | All users, case-insensitive email lookup |
| `rooms` | Room types and rooms | All records, types by hotel, rooms by type or hotel |
| `pricing` | Price plans | All plans, plans by room type |
| `bookings` | Bookings | All bookings, bookings by user or room |

Each record has separate `Create*` and `Update*` inputs. Updates replace all
editable fields, including setting nullable descriptions to `NULL` with `None`.
Lists take `limit` and `offset` and use stable ordering by ID. The service must
validate pagination (non-negative offset, bounded non-negative limit).

`auth` uses the user repository for persistence; JWT claims have no database table.
User inputs accept password hashes, so hashing and role authorization belong in
the service. Persistence models contain sensitive fields and should not be
returned directly from authentication handlers.

New bookings use the database default `pending` status. The booking repository
also supports updating just the status. Per the current layer separation, the
service must enforce availability, capacity, price-plan compatibility, pricing,
ownership and allowed status transitions. A separate availability check followed
by insertion is insufficient under concurrency; the future booking workflow
needs a transaction with appropriate locking or a database exclusion constraint.
Room/type/plan reassignment must also preserve existing booking relationships.

## Repository tests

With the local PostgreSQL container running and `DATABASE_URL` in `.env`:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

SQLx loads `.env` and creates a separate database per test, runs the migrations,
and deletes successful test databases. The database user needs `CREATEDB`.
SQLx keeps test bookkeeping in the `_sqlx_test` schema of the connection database;
fixtures do not modify the application's business tables. Failed test databases
may be retained for inspection.
