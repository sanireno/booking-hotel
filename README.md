# Booking Hotel API

Backend service for a hotel booking system written in Rust with Axum, SQLx, and PostgreSQL.

## Local setup

Requirements: Rust and PostgreSQL 13+ (or Docker with Docker Compose).

```bash
cp .env.example .env
# Set JWT_SECRET in .env to a random secret generated with:
openssl rand -hex 32
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

## Authentication

Authentication is implemented in `src/auth`: HTTP DTOs, handlers, services,
Argon2id password hashing, JWT issuing/validation and an `AuthUser` extractor.
Shared `AppState` holds the database pool, token service and password service.

| Method | Path | Result |
| --- | --- | --- |
| POST | `/auth/register` | `201` with an access token and public user fields |
| POST | `/auth/login` | `200` with an access token and public user fields |
| GET | `/auth/me` | `200` with the authenticated user's public fields |

Registration body:

```json
{
  "email": "customer@example.com",
  "password": "correct horse battery staple",
  "first_name": "Test",
  "last_name": "User"
}
```

Login accepts only `email` and `password`. Registration always assigns the
`customer` role; unknown fields such as `role` or `password_hash` are rejected.
Passwords must contain 12 to 128 characters and are preserved exactly, including
spaces. Names are trimmed, must contain 1 to 100 characters and cannot contain
control characters. Email addresses are trimmed and ASCII-lowercased, with
case-insensitive lookup and uniqueness in PostgreSQL.

Successful registration/login returns:

```json
{
  "access_token": "<signed JWT>",
  "token_type": "Bearer",
  "expires_in": 3600,
  "user": {
    "id": "<UUID>",
    "email": "customer@example.com",
    "first_name": "Test",
    "last_name": "User",
    "role": "customer",
    "created_at": "<UTC timestamp>"
  }
}
```

Pass the token in `Authorization: Bearer <access_token>` to `/auth/me` and any
future protected endpoint. Add `AuthUser` before the body extractor in a handler:

```rust
use crate::auth::{dto::UserResponse, extractor::AuthUser};
use axum::Json;

async fn protected(AuthUser(user): AuthUser) -> Json<UserResponse> {
    // user.id and user.role are loaded from the current database record.
    Json(user)
}
```

For an exact role check, keep the wrapper and call
`auth.require_role(UserRole::Admin)?`. For more involved rules, pass the
authenticated user to the service and check ownership or permitted roles there.
Use routers with `AppState`; changing a user's role takes effect on the next
request and a deleted user can no longer authenticate with an existing token.

JWTs use HS256 and validate the signature, algorithm, expiration, not-before
time, issuer and audience. `JWT_SECRET` is required and must contain at least
32 bytes; use a randomly generated value. `JWT_TTL_SECONDS` defaults to `3600`
and accepts `1..=86400`. No secret is embedded in the source code, and configuration
does not derive `Debug`. Rotating the secret invalidates existing tokens.
Access tokens expire naturally; refresh tokens and per-token server-side logout
are not part of this implementation.

Passwords use a random salt per hash. Hashing/verification runs in blocking
workers with at most two concurrent jobs per application state. Login with an
unknown account also verifies a dummy hash and returns the same `401` response
as a wrong password. Authentication responses use `Cache-Control: no-store`;
passwords and hashes are excluded from response DTOs. Auth request bodies are
limited to 16 KiB.

Errors use the existing JSON shape `{"error":"..."}`: `400` for malformed JSON,
`415` for a missing/incorrect JSON content type, `413` for oversized bodies,
`422` for invalid fields, `409` for an existing email, `401` for failed authentication,
and `403` for a failed role check. `401` responses include `WWW-Authenticate: Bearer`.

Tests exercise these routes against isolated PostgreSQL databases, including
password hashing, validation, duplicate emails, forged/expired JWTs, claim
validation, deleted users, role changes and the shared health endpoint.

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
