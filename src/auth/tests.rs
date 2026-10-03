use super::{dto::RegisterRequest, extractor::AuthUser, service, token::TokenService};
use crate::{
    app_error::AppError,
    app_state::AppState,
    config::AuthConfig,
    users::{
        model::{UpdateUser, UserRole},
        repository,
    },
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{
        HeaderMap, Method, Request, StatusCode,
        header::{AUTHORIZATION, CACHE_CONTROL, CONTENT_TYPE, WWW_AUTHENTICATE},
    },
    routing::get,
};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, get_current_timestamp};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &str = "test-only-secret-with-at-least-32-bytes";
const PASSWORD: &str = " correct horse battery staple ";

fn config() -> AuthConfig {
    AuthConfig {
        jwt_secret: SECRET.into(),
        jwt_ttl_seconds: 3600,
    }
}

async fn state(pool: PgPool) -> Result<AppState, AppError> {
    AppState::new(pool, &config()).await
}

fn registration(email: &str) -> Value {
    json!({"email": email, "password": PASSWORD, "first_name": " Test ", "last_name": " User "})
}

async fn register_user(state: &AppState) -> Result<super::dto::AuthResponse, AppError> {
    service::register(
        state,
        RegisterRequest {
            email: "customer@example.com".into(),
            password: PASSWORD.into(),
            first_name: "Test".into(),
            last_name: "User".into(),
        },
    )
    .await
}

async fn request(
    app: &Router,
    method: Method,
    path: &str,
    body: Option<Value>,
    authorization: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(authorization) = authorization {
        builder = builder.header(AUTHORIZATION, authorization);
    }
    let body = if let Some(body) = body {
        builder = builder.header(CONTENT_TYPE, "application/json");
        Body::from(serde_json::to_vec(&body).unwrap())
    } else {
        Body::empty()
    };
    let response = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, headers, body)
}

#[sqlx::test(migrations = "./migrations")]
async fn register_login_and_me(pool: PgPool) -> Result<(), AppError> {
    let state = state(pool).await?;
    let app = crate::app(state.clone(), &crate::config::CorsConfig::default());
    let (status, headers, registered) = request(
        &app,
        Method::POST,
        "/auth/register",
        Some(registration(" Customer@Example.com ")),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(headers[CACHE_CONTROL], "no-store");
    assert_eq!(registered["user"]["email"], "customer@example.com");
    assert_eq!(registered["user"]["first_name"], "Test");
    assert_eq!(registered["user"]["last_name"], "User");
    assert_eq!(registered["user"]["role"], "customer");
    assert_eq!(registered["token_type"], "Bearer");
    assert_eq!(registered["expires_in"], 3600);
    assert!(!registered.to_string().contains("password"));
    let stored = repository::get_user_by_email(&state.pool, "customer@example.com").await?;
    assert!(stored.password_hash.starts_with("$argon2id$"));
    assert_ne!(stored.password_hash, PASSWORD);
    assert!(
        state
            .passwords
            .verify(PASSWORD.into(), Some(stored.password_hash.clone()))
            .await?
    );
    let token = registered["access_token"].as_str().unwrap();
    let claims = state.tokens.verify(token)?;
    assert_eq!(claims.sub, stored.id);
    assert_eq!(claims.role, UserRole::Customer);
    assert_eq!(claims.exp - claims.iat, 3600);
    let bearer = format!("Bearer {token}");
    let (status, headers, me) = request(&app, Method::GET, "/auth/me", None, Some(&bearer)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[CACHE_CONTROL], "no-store");
    assert_eq!(me, registered["user"]);

    let (status, _, _) = request(
        &app,
        Method::POST,
        "/auth/register",
        Some(registration("CUSTOMER@example.COM")),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, headers, logged_in) = request(
        &app,
        Method::POST,
        "/auth/login",
        Some(json!({"email": " CUSTOMER@example.COM ", "password": PASSWORD})),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[CACHE_CONTROL], "no-store");
    assert_eq!(logged_in["user"], registered["user"]);
    assert!(!logged_in.to_string().contains("password"));
    assert_eq!(
        state
            .tokens
            .verify(logged_in["access_token"].as_str().unwrap())?
            .sub,
        stored.id
    );

    let (status, _, _) = request(
        &app,
        Method::POST,
        "/auth/register",
        Some(registration("second@example.com")),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let second = repository::get_user_by_email(&state.pool, "second@example.com").await?;
    assert_ne!(second.password_hash, stored.password_hash);
    let (status, _, health) = request(&app, Method::GET, "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(health, json!({"status": "ok", "database": "up"}));
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn wrong_password_and_unknown_user_have_same_response(pool: PgPool) -> Result<(), AppError> {
    let state = state(pool).await?;
    register_user(&state).await?;
    let app = crate::app(state, &crate::config::CorsConfig::default());
    let mut errors = Vec::new();
    for (email, password) in [
        ("customer@example.com", "wrong-password"),
        ("missing@example.com", PASSWORD),
        ("customer@example.com", PASSWORD.trim()),
        ("missing@example.com", "dummy-account-password"),
        ("customer@example.com", ""),
    ] {
        let (status, headers, body) = request(
            &app,
            Method::POST,
            "/auth/login",
            Some(json!({"email": email, "password": password})),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(headers[WWW_AUTHENTICATE], "Bearer");
        errors.push(body);
    }
    assert!(errors.iter().all(|body| body == &errors[0]));
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn invalid_registration_and_json_are_rejected(pool: PgPool) -> Result<(), AppError> {
    let state = state(pool).await?;
    let app = crate::app(state.clone(), &crate::config::CorsConfig::default());
    let mut invalid = Vec::new();
    for (field, value) in [
        ("email", json!("invalid-email")),
        ("email", json!("Name <customer@example.com>")),
        ("password", json!("short")),
        ("password", json!(" ".repeat(20))),
        ("password", json!("x".repeat(129))),
        ("first_name", json!("   ")),
        ("last_name", json!("line\nbreak")),
        ("role", json!("admin")),
        ("password_hash", json!("injected-hash")),
    ] {
        let mut body = registration("customer@example.com");
        body[field] = value;
        invalid.push(body);
    }
    invalid.push(json!({"email": "customer@example.com", "password": PASSWORD}));
    for body in invalid {
        let (status, _, body) =
            request(&app, Method::POST, "/auth/register", Some(body), None).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body["error"].is_string());
    }
    let mut oversized = registration("customer@example.com");
    oversized["first_name"] = json!("x".repeat(17 * 1024));
    assert_eq!(
        request(&app, Method::POST, "/auth/register", Some(oversized), None)
            .await
            .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    let missing_type = Request::builder()
        .method(Method::POST)
        .uri("/auth/login")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(missing_type).await.unwrap().status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
    let malformed = Request::builder()
        .method(Method::POST)
        .uri("/auth/login")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from("{"))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(malformed).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await?;
    assert_eq!(count, 0);
    Ok(())
}

fn sign<T: serde::Serialize>(claims: &T, algorithm: Algorithm, secret: &str) -> String {
    encode(
        &Header::new(algorithm),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn extractor_rejects_invalid_tokens_and_headers(pool: PgPool) -> Result<(), AppError> {
    let state = state(pool).await?;
    let registered = register_user(&state).await?;
    let claims = state.tokens.verify(&registered.access_token)?;
    let app = crate::app(state, &crate::config::CorsConfig::default());
    for header in [
        None,
        Some("Basic abc"),
        Some("Bearer"),
        Some("Bearer garbage"),
        Some("Bearer token extra"),
    ] {
        let (status, headers, _) = request(&app, Method::GET, "/auth/me", None, header).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(headers[WWW_AUTHENTICATE], "Bearer");
    }
    let mut invalid_tokens = vec![
        sign(
            &claims,
            Algorithm::HS256,
            "different-test-secret-with-at-least-32-bytes",
        ),
        sign(&claims, Algorithm::HS384, SECRET),
        "eyJhbGciOiJub25lIn0.e30.".into(),
    ];
    let mut expired = claims.clone();
    expired.exp = get_current_timestamp() - 1;
    invalid_tokens.push(sign(&expired, Algorithm::HS256, SECRET));
    let mut future = claims.clone();
    future.nbf = get_current_timestamp() + 3600;
    invalid_tokens.push(sign(&future, Algorithm::HS256, SECRET));
    let mut wrong_issuer = claims.clone();
    wrong_issuer.iss = "other-service".into();
    invalid_tokens.push(sign(&wrong_issuer, Algorithm::HS256, SECRET));
    let mut wrong_audience = claims.clone();
    wrong_audience.aud = "other-api".into();
    invalid_tokens.push(sign(&wrong_audience, Algorithm::HS256, SECRET));
    let mut missing_user = claims.clone();
    missing_user.sub = Uuid::new_v4();
    invalid_tokens.push(sign(&missing_user, Algorithm::HS256, SECRET));
    for field in ["sub", "exp", "nbf", "iss", "aud", "role", "iat"] {
        let mut missing = serde_json::to_value(&claims).unwrap();
        missing.as_object_mut().unwrap().remove(field);
        invalid_tokens.push(sign(&missing, Algorithm::HS256, SECRET));
    }
    let mut invalid_sub = serde_json::to_value(&claims).unwrap();
    invalid_sub["sub"] = json!("invalid-uuid");
    invalid_tokens.push(sign(&invalid_sub, Algorithm::HS256, SECRET));
    for token in invalid_tokens {
        let header = format!("Bearer {token}");
        assert_eq!(
            request(&app, Method::GET, "/auth/me", None, Some(&header))
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
    }
    let valid_header = format!("bearer {}", registered.access_token);
    assert_eq!(
        request(&app, Method::GET, "/auth/me", None, Some(&valid_header))
            .await
            .0,
        StatusCode::OK
    );
    let mut duplicate = Request::builder()
        .uri("/auth/me")
        .header(AUTHORIZATION, &valid_header)
        .body(Body::empty())
        .unwrap();
    duplicate
        .headers_mut()
        .append(AUTHORIZATION, valid_header.parse().unwrap());
    assert_eq!(
        app.oneshot(duplicate).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    Ok(())
}

async fn admin_only(auth: AuthUser) -> Result<StatusCode, AppError> {
    auth.require_role(UserRole::Admin)?;
    Ok(StatusCode::NO_CONTENT)
}

#[sqlx::test(migrations = "./migrations")]
async fn database_role_changes_and_deleted_accounts_are_respected(
    pool: PgPool,
) -> Result<(), AppError> {
    let state = state(pool).await?;
    let registered = register_user(&state).await?;
    let bearer = format!("Bearer {}", registered.access_token);
    let app = Router::new()
        .route("/admin", get(admin_only))
        .nest("/auth", super::routes())
        .with_state(state.clone());
    assert_eq!(
        request(&app, Method::GET, "/admin", None, Some(&bearer))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    let original = repository::get_user(&state.pool, registered.user.id).await?;
    for (role, expected_status) in [
        (UserRole::Admin, StatusCode::NO_CONTENT),
        (UserRole::Customer, StatusCode::FORBIDDEN),
    ] {
        repository::update_user(
            &state.pool,
            original.id,
            UpdateUser {
                email: original.email.clone(),
                password_hash: original.password_hash.clone(),
                first_name: original.first_name.clone(),
                last_name: original.last_name.clone(),
                role,
            },
        )
        .await?;
        assert_eq!(
            request(&app, Method::GET, "/admin", None, Some(&bearer))
                .await
                .0,
            expected_status
        );
        let me = request(&app, Method::GET, "/auth/me", None, Some(&bearer))
            .await
            .2;
        assert_eq!(me["role"], serde_json::to_value(role).unwrap());
    }
    repository::delete_user(&state.pool, original.id).await?;
    assert_eq!(
        request(&app, Method::GET, "/auth/me", None, Some(&bearer))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    Ok(())
}

#[test]
fn token_configuration_requires_a_secret_and_bounded_lifetime() {
    let mut config = config();
    config.jwt_secret = "short".into();
    assert!(TokenService::new(&config).is_err());
    config.jwt_secret = " ".repeat(32);
    assert!(TokenService::new(&config).is_err());
    config.jwt_secret = SECRET.into();
    for ttl in [0, 86401, u64::MAX] {
        config.jwt_ttl_seconds = ttl;
        assert!(TokenService::new(&config).is_err());
    }
    config.jwt_ttl_seconds = 3600;
    assert!(TokenService::new(&config).is_ok());
}
