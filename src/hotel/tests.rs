use crate::{
    app_error::AppError,
    app_state::AppState,
    config::AuthConfig,
    hotel::{model::CreateHotel, repository},
    users::{
        model::{CreateUser, UpdateUser, User, UserRole},
        repository as users,
    },
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{
        HeaderMap, Method, Request, StatusCode,
        header::{AUTHORIZATION, CONTENT_TYPE, LOCATION},
    },
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

async fn state(pool: PgPool) -> Result<AppState, AppError> {
    AppState::new(
        pool,
        &AuthConfig {
            jwt_secret: "hotel-test-only-secret-with-at-least-32-bytes".into(),
            jwt_ttl_seconds: 3600,
        },
    )
    .await
}

async fn actor(state: &AppState, role: UserRole) -> Result<(User, String), AppError> {
    let user = users::create_user(
        &state.pool,
        CreateUser {
            email: format!("{}@example.com", Uuid::new_v4()),
            password_hash: "unused-test-hash".into(),
            first_name: "Test".into(),
            last_name: "Actor".into(),
            role,
        },
    )
    .await?;
    let bearer = format!("Bearer {}", state.tokens.issue(&user)?);
    Ok((user, bearer))
}

fn payload() -> Value {
    json!({"name": " Test Hotel ", "description": " Description ", "city": " Irkutsk ", "address": " Street 1 "})
}

async fn request(
    app: &Router,
    method: Method,
    path: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        builder = builder.header(AUTHORIZATION, token);
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
async fn hotel_crud_and_full_updates(pool: PgPool) -> Result<(), AppError> {
    let state = state(pool).await?;
    let (_, admin) = actor(&state, UserRole::Admin).await?;
    let app = crate::app(state);
    let (status, _, empty) = request(&app, Method::GET, "/hotels", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(empty, json!([]));
    let (status, headers, created) =
        request(&app, Method::POST, "/hotels", Some(payload()), Some(&admin)).await;
    assert_eq!(status, StatusCode::CREATED);
    let id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
    let path = format!("/hotels/{id}");
    assert_eq!(headers[LOCATION], path);
    assert_eq!(created["name"], "Test Hotel");
    assert_eq!(created["description"], "Description");
    assert_eq!(created["city"], "Irkutsk");
    assert_eq!(created["address"], "Street 1");
    assert!(created["created_at"].is_string());
    let (status, _, fetched) = request(&app, Method::GET, &path, None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(fetched, created);
    let update = json!({"name": " New Hotel ", "city": " Angarsk ", "address": " Street 2 "});
    let (status, _, updated) = request(&app, Method::PUT, &path, Some(update), Some(&admin)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["id"], created["id"]);
    assert_eq!(updated["created_at"], created["created_at"]);
    assert_eq!(updated["name"], "New Hotel");
    assert_eq!(updated["description"], Value::Null);
    assert_eq!(updated["city"], "Angarsk");
    assert_eq!(updated["address"], "Street 2");
    assert_eq!(
        request(&app, Method::GET, &path, None, None).await.2,
        updated
    );
    let mut update = payload();
    update["description"] = json!("   ");
    let updated = request(&app, Method::PUT, &path, Some(update), Some(&admin)).await;
    assert_eq!(updated.0, StatusCode::OK);
    assert_eq!(updated.2["description"], Value::Null);
    let (status, _, body) = request(&app, Method::DELETE, &path, None, Some(&admin)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(body, Value::Null);
    for method in [Method::GET, Method::PUT, Method::DELETE] {
        let body = (method == Method::PUT).then(payload);
        let (status, _, body) = request(&app, method, &path, body, Some(&admin)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, json!({"error": "Not found"}));
    }
    let (status, _, body) = request(&app, Method::GET, "/hotels/not-a-uuid", None, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].is_string());
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn hotel_list_is_public_and_pagination_is_bounded(pool: PgPool) -> Result<(), AppError> {
    let state = state(pool).await?;
    for index in 0..25 {
        repository::create_hotel(
            &state.pool,
            CreateHotel {
                name: format!("Hotel {index}"),
                description: None,
                city: "Irkutsk".into(),
                address: "Street 1".into(),
            },
        )
        .await?;
    }
    let app = crate::app(state);
    let (status, _, default_page) = request(&app, Method::GET, "/hotels", None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(default_page.as_array().unwrap().len(), 20);
    let all = request(&app, Method::GET, "/hotels?limit=100", None, None)
        .await
        .2;
    assert_eq!(all.as_array().unwrap().len(), 25);
    assert_eq!(
        default_page.as_array().unwrap(),
        &all.as_array().unwrap()[..20]
    );
    assert!(
        all.as_array()
            .unwrap()
            .windows(2)
            .all(|pair| pair[0]["id"].as_str().unwrap() < pair[1]["id"].as_str().unwrap())
    );
    let page = request(&app, Method::GET, "/hotels?limit=2&offset=3", None, None).await;
    assert_eq!(page.0, StatusCode::OK);
    assert_eq!(page.2.as_array().unwrap(), &all.as_array().unwrap()[3..5]);
    assert_eq!(
        request(&app, Method::GET, "/hotels?offset=25", None, None)
            .await
            .2,
        json!([])
    );
    for query in ["limit=0", "limit=-1", "limit=101", "offset=-1"] {
        let (status, _, body) =
            request(&app, Method::GET, &format!("/hotels?{query}"), None, None).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body["error"].is_string());
    }
    for query in ["limit=invalid", "offset=9223372036854775808", "unknown=1"] {
        let (status, _, body) =
            request(&app, Method::GET, &format!("/hotels?{query}"), None, None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body["error"].is_string());
    }
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn hotel_writes_require_the_current_admin_role(pool: PgPool) -> Result<(), AppError> {
    let state = state(pool).await?;
    let hotel = repository::create_hotel(
        &state.pool,
        CreateHotel {
            name: "Original".into(),
            description: None,
            city: "Irkutsk".into(),
            address: "Street 1".into(),
        },
    )
    .await?;
    let path = format!("/hotels/{}", hotel.id);
    let app = crate::app(state.clone());
    for token in [None, Some("Bearer invalid")] {
        for method in [Method::POST, Method::PUT, Method::DELETE] {
            let endpoint = if method == Method::POST {
                "/hotels"
            } else {
                &path
            };
            let body = (method != Method::DELETE).then(payload);
            assert_eq!(
                request(&app, method, endpoint, body, token).await.0,
                StatusCode::UNAUTHORIZED
            );
        }
    }
    for role in [UserRole::Customer, UserRole::Manager] {
        let (_, token) = actor(&state, role).await?;
        for method in [Method::POST, Method::PUT, Method::DELETE] {
            let endpoint = if method == Method::POST {
                "/hotels"
            } else {
                &path
            };
            let body = (method != Method::DELETE).then(payload);
            assert_eq!(
                request(&app, method, endpoint, body, Some(&token)).await.0,
                StatusCode::FORBIDDEN
            );
        }
    }
    assert_eq!(
        repository::get_all_hotels(&state.pool, 100, 0).await?.len(),
        1
    );
    assert_eq!(
        repository::get_hotel(&state.pool, hotel.id).await?.name,
        "Original"
    );
    let (admin, token) = actor(&state, UserRole::Admin).await?;
    users::update_user(
        &state.pool,
        admin.id,
        UpdateUser {
            email: admin.email,
            password_hash: admin.password_hash,
            first_name: admin.first_name,
            last_name: admin.last_name,
            role: UserRole::Customer,
        },
    )
    .await?;
    assert_eq!(
        request(&app, Method::PUT, &path, Some(payload()), Some(&token))
            .await
            .0,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        repository::get_hotel(&state.pool, hotel.id).await?.name,
        "Original"
    );
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn hotel_payload_validation_and_http_errors(pool: PgPool) -> Result<(), AppError> {
    let state = state(pool).await?;
    let (_, admin) = actor(&state, UserRole::Admin).await?;
    let app = crate::app(state.clone());
    let created = request(&app, Method::POST, "/hotels", Some(payload()), Some(&admin))
        .await
        .2;
    let path = format!("/hotels/{}", created["id"].as_str().unwrap());
    for (field, value) in [
        ("name", json!("   ")),
        ("name", json!("x".repeat(201))),
        ("name", json!("line\nbreak")),
        ("city", json!("   ")),
        ("city", json!("x".repeat(101))),
        ("address", json!("")),
        ("address", json!("x".repeat(501))),
        ("description", json!("x".repeat(5001))),
        ("description", json!("text\u{0}value")),
        ("description", json!(false)),
        ("id", json!(Uuid::new_v4())),
        ("created_at", json!("2026-01-01")),
    ] {
        let mut invalid = payload();
        invalid[field] = value;
        for method in [Method::POST, Method::PUT] {
            let endpoint = if method == Method::POST {
                "/hotels"
            } else {
                &path
            };
            let (status, _, body) =
                request(&app, method, endpoint, Some(invalid.clone()), Some(&admin)).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
            assert!(body["error"].is_string());
        }
    }
    assert_eq!(
        request(
            &app,
            Method::PUT,
            &path,
            Some(json!({"name": "Partial"})),
            Some(&admin)
        )
        .await
        .0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        repository::get_all_hotels(&state.pool, 100, 0).await?.len(),
        1
    );
    assert_eq!(
        request(&app, Method::GET, &path, None, None).await.2,
        created
    );
    let mut valid = payload();
    valid["name"] = json!("\u{041e}".repeat(200));
    valid["city"] = json!("x".repeat(100));
    valid["address"] = json!("x".repeat(500));
    valid["description"] = json!(" first line\nsecond line\t ");
    let (status, _, body) = request(&app, Method::POST, "/hotels", Some(valid), Some(&admin)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["name"].as_str().unwrap().chars().count(), 200);
    assert_eq!(body["description"], "first line\nsecond line");
    let mut oversized = payload();
    oversized["description"] = json!("x".repeat(33 * 1024));
    assert_eq!(
        request(&app, Method::POST, "/hotels", Some(oversized), Some(&admin))
            .await
            .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    let malformed = Request::builder()
        .method(Method::POST)
        .uri("/hotels")
        .header(AUTHORIZATION, &admin)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from("{"))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(malformed).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );
    let missing_type = Request::builder()
        .method(Method::POST)
        .uri("/hotels")
        .header(AUTHORIZATION, &admin)
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(
        app.oneshot(missing_type).await.unwrap().status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn hotel_with_bookings_cannot_be_deleted(pool: PgPool) -> Result<(), AppError> {
    use crate::{
        bookings::{model::CreateBooking, repository as bookings},
        pricing::{model::CreatePricePlan, repository as pricing},
        rooms::{
            model::{CreateRoom, CreateRoomType},
            repository as rooms,
        },
    };
    let state = state(pool).await?;
    let (admin_user, admin) = actor(&state, UserRole::Admin).await?;
    let app = crate::app(state.clone());
    let created = request(&app, Method::POST, "/hotels", Some(payload()), Some(&admin))
        .await
        .2;
    let hotel_id = Uuid::parse_str(created["id"].as_str().unwrap()).unwrap();
    let path = format!("/hotels/{hotel_id}");
    let room_type = rooms::create_room_type(
        &state.pool,
        CreateRoomType {
            hotel_id,
            name: "Standard".into(),
            description: None,
            capacity: 2,
            base_price: Decimal::new(1000, 2),
        },
    )
    .await?;
    let room = rooms::create_room(
        &state.pool,
        CreateRoom {
            room_type_id: room_type.id,
            number: "101".into(),
        },
    )
    .await?;
    let plan = pricing::create_price_plan(
        &state.pool,
        CreatePricePlan {
            room_type_id: room_type.id,
            name: "Standard".into(),
            price_per_night: Decimal::new(1000, 2),
            refundable: false,
        },
    )
    .await?;
    let booking = bookings::create_booking(
        &state.pool,
        CreateBooking {
            user_id: admin_user.id,
            room_id: room.id,
            price_plan_id: plan.id,
            check_in: NaiveDate::from_ymd_opt(2026, 10, 10).unwrap(),
            check_out: NaiveDate::from_ymd_opt(2026, 10, 11).unwrap(),
            guests: 1,
            total_price: Decimal::new(1000, 2),
        },
    )
    .await?;
    let (status, _, body) = request(&app, Method::DELETE, &path, None, Some(&admin)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body,
        json!({"error": "Hotel has bookings and cannot be deleted"})
    );
    assert_eq!(
        request(&app, Method::GET, &path, None, None).await.0,
        StatusCode::OK
    );
    assert_eq!(
        rooms::get_room(&state.pool, room.id).await?.room_type_id,
        room_type.id
    );
    assert_eq!(
        pricing::get_price_plan(&state.pool, plan.id)
            .await?
            .room_type_id,
        room_type.id
    );
    assert_eq!(
        bookings::get_booking(&state.pool, booking.id)
            .await?
            .room_id,
        room.id
    );
    bookings::delete_booking(&state.pool, booking.id).await?;
    assert_eq!(
        request(&app, Method::DELETE, &path, None, Some(&admin))
            .await
            .0,
        StatusCode::NO_CONTENT
    );
    assert!(matches!(
        rooms::get_room(&state.pool, room.id).await,
        Err(sqlx::Error::RowNotFound)
    ));
    assert!(matches!(
        rooms::get_room_type(&state.pool, room_type.id).await,
        Err(sqlx::Error::RowNotFound)
    ));
    assert!(matches!(
        pricing::get_price_plan(&state.pool, plan.id).await,
        Err(sqlx::Error::RowNotFound)
    ));
    Ok(())
}
