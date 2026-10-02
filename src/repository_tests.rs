use crate::{bookings, hotel, pricing, rooms, users};
use bookings::model::{CreateBooking, Status, UpdateBooking};
use chrono::NaiveDate;
use hotel::model::{CreateHotel, Hotel, UpdateHotel};
use pricing::model::{CreatePricePlan, PricePlan, UpdatePricePlan};
use rooms::model::{CreateRoom, CreateRoomType, Room, RoomType, UpdateRoom, UpdateRoomType};
use rust_decimal::Decimal;
use sqlx::PgPool;
use users::model::{CreateUser, UpdateUser, User, UserRole};
use uuid::Uuid;

async fn create_user(pool: &PgPool, email: &str) -> Result<User, sqlx::Error> {
    users::repository::create_user(
        pool,
        CreateUser {
            email: email.into(),
            password_hash: "test-hash".into(),
            first_name: "Test".into(),
            last_name: "User".into(),
            role: UserRole::Customer,
        },
    )
    .await
}

struct Fixture {
    user: User,
    hotel: Hotel,
    room_type: RoomType,
    room: Room,
    plan: PricePlan,
}

impl Fixture {
    async fn create(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let user = create_user(pool, &format!("{}@example.com", Uuid::new_v4())).await?;
        let hotel = hotel::repository::create_hotel(
            pool,
            CreateHotel {
                name: "Test Hotel".into(),
                description: Some("Description".into()),
                city: "Irkutsk".into(),
                address: "Test Street 1".into(),
            },
        )
        .await?;
        let room_type = rooms::repository::create_room_type(
            pool,
            CreateRoomType {
                hotel_id: hotel.id,
                name: "Double".into(),
                description: Some("Two guests".into()),
                capacity: 2,
                base_price: Decimal::new(12345, 2),
            },
        )
        .await?;
        let room = rooms::repository::create_room(
            pool,
            CreateRoom {
                room_type_id: room_type.id,
                number: "101".into(),
            },
        )
        .await?;
        let plan = pricing::repository::create_price_plan(
            pool,
            CreatePricePlan {
                room_type_id: room_type.id,
                name: "Standard".into(),
                price_per_night: Decimal::new(12345, 2),
                refundable: false,
            },
        )
        .await?;
        Ok(Self {
            user,
            hotel,
            room_type,
            room,
            plan,
        })
    }

    fn booking(&self) -> CreateBooking {
        CreateBooking {
            user_id: self.user.id,
            room_id: self.room.id,
            price_plan_id: self.plan.id,
            check_in: NaiveDate::from_ymd_opt(2026, 10, 10).unwrap(),
            check_out: NaiveDate::from_ymd_opt(2026, 10, 12).unwrap(),
            guests: 2,
            total_price: Decimal::new(24690, 2),
        }
    }
}

fn assert_database_code(error: sqlx::Error, code: &str) {
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some(code)
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn users_crud_and_case_insensitive_email(pool: PgPool) -> Result<(), sqlx::Error> {
    let user = create_user(&pool, "Customer@Example.com").await?;
    assert_eq!(user.role, UserRole::Customer);
    assert_eq!(
        users::repository::get_user(&pool, user.id).await?.email,
        user.email
    );
    assert_eq!(
        users::repository::get_user_by_email(&pool, "customer@EXAMPLE.COM")
            .await?
            .id,
        user.id
    );
    assert_database_code(
        create_user(&pool, "customer@example.COM")
            .await
            .unwrap_err(),
        "23505",
    );

    for role in [UserRole::Manager, UserRole::Admin] {
        let updated = users::repository::update_user(
            &pool,
            user.id,
            UpdateUser {
                email: "updated@example.com".into(),
                password_hash: "new-hash".into(),
                first_name: "Updated".into(),
                last_name: "Name".into(),
                role,
            },
        )
        .await?;
        assert_eq!(updated.role, role);
        assert_eq!(updated.password_hash, "new-hash");
        assert_eq!(updated.first_name, "Updated");
        assert_eq!(updated.last_name, "Name");
        assert_eq!(updated.created_at, user.created_at);
        assert_eq!(
            users::repository::get_user(&pool, user.id).await?.role,
            role
        );
    }
    create_user(&pool, "other@example.com").await?;
    let all = users::repository::get_all_users(&pool, 10, 0).await?;
    let page = users::repository::get_all_users(&pool, 1, 1).await?;
    assert_eq!(all.len(), 2);
    assert_eq!(page[0].id, all[1].id);
    assert!(
        users::repository::get_all_users(&pool, 1, 2)
            .await?
            .is_empty()
    );
    assert_eq!(
        users::repository::delete_user(&pool, user.id).await?.id,
        user.id
    );
    assert!(matches!(
        users::repository::get_user(&pool, user.id).await,
        Err(sqlx::Error::RowNotFound)
    ));
    assert!(matches!(
        users::repository::get_user_by_email(&pool, "missing@example.com").await,
        Err(sqlx::Error::RowNotFound)
    ));
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn hotels_rooms_and_pricing_crud_and_scoped_lists(pool: PgPool) -> Result<(), sqlx::Error> {
    let f = Fixture::create(&pool).await?;
    let other = Fixture::create(&pool).await?;
    assert_eq!(
        hotel::repository::get_hotel(&pool, f.hotel.id).await?.name,
        "Test Hotel"
    );
    let hotel = hotel::repository::update_hotel(
        &pool,
        f.hotel.id,
        UpdateHotel {
            name: "Updated Hotel".into(),
            description: None,
            city: "Angarsk".into(),
            address: "Street 2".into(),
        },
    )
    .await?;
    assert_eq!(hotel.name, "Updated Hotel");
    assert!(hotel.description.is_none());
    assert_eq!(hotel.city, "Angarsk");
    assert_eq!(hotel.address, "Street 2");
    assert_eq!(hotel.created_at, f.hotel.created_at);
    let all = hotel::repository::get_all_hotels(&pool, 10, 0).await?;
    assert_eq!(all.len(), 2);
    assert_eq!(
        hotel::repository::get_all_hotels(&pool, 1, 1).await?[0].id,
        all[1].id
    );

    assert_eq!(
        rooms::repository::get_room_type(&pool, f.room_type.id)
            .await?
            .base_price,
        Decimal::new(12345, 2)
    );
    let rt = rooms::repository::update_room_type(
        &pool,
        f.room_type.id,
        UpdateRoomType {
            hotel_id: other.hotel.id,
            name: "Family".into(),
            description: None,
            capacity: 4,
            base_price: Decimal::new(20099, 2),
        },
    )
    .await?;
    assert_eq!(rt.hotel_id, other.hotel.id);
    assert_eq!(rt.name, "Family");
    assert!(rt.description.is_none());
    assert_eq!(rt.capacity, 4);
    assert_eq!(rt.base_price, Decimal::new(20099, 2));
    assert!(
        rooms::repository::get_room_types_by_hotel(&pool, f.hotel.id, 10, 0)
            .await?
            .is_empty()
    );
    let scoped = rooms::repository::get_room_types_by_hotel(&pool, other.hotel.id, 10, 0).await?;
    assert_eq!(scoped.len(), 2);
    assert_eq!(
        rooms::repository::get_room_types_by_hotel(&pool, other.hotel.id, 1, 1).await?[0].id,
        scoped[1].id
    );
    let all = rooms::repository::get_all_room_types(&pool, 10, 0).await?;
    assert_eq!(all.len(), 2);
    assert_eq!(
        rooms::repository::get_all_room_types(&pool, 1, 1).await?[0].id,
        all[1].id
    );

    assert_eq!(
        rooms::repository::get_room(&pool, f.room.id).await?.number,
        "101"
    );
    let room = rooms::repository::update_room(
        &pool,
        f.room.id,
        UpdateRoom {
            room_type_id: other.room_type.id,
            number: "202".into(),
        },
    )
    .await?;
    assert_eq!(room.number, "202");
    assert_eq!(room.room_type_id, other.room_type.id);
    assert!(
        rooms::repository::get_rooms_by_room_type(&pool, f.room_type.id, 10, 0)
            .await?
            .is_empty()
    );
    let scoped =
        rooms::repository::get_rooms_by_room_type(&pool, other.room_type.id, 10, 0).await?;
    assert_eq!(scoped.len(), 2);
    assert_eq!(
        rooms::repository::get_rooms_by_room_type(&pool, other.room_type.id, 1, 1).await?[0].id,
        scoped[1].id
    );
    assert!(
        rooms::repository::get_rooms_by_hotel(&pool, f.hotel.id, 10, 0)
            .await?
            .is_empty()
    );
    assert_eq!(
        rooms::repository::get_rooms_by_hotel(&pool, other.hotel.id, 1, 1).await?[0].id,
        scoped[1].id
    );
    let all = rooms::repository::get_all_rooms(&pool, 10, 0).await?;
    assert_eq!(all.len(), 2);
    assert_eq!(
        rooms::repository::get_all_rooms(&pool, 1, 1).await?[0].id,
        all[1].id
    );

    assert_eq!(
        pricing::repository::get_price_plan(&pool, f.plan.id)
            .await?
            .price_per_night,
        Decimal::new(12345, 2)
    );
    let plan = pricing::repository::update_price_plan(
        &pool,
        f.plan.id,
        UpdatePricePlan {
            room_type_id: other.room_type.id,
            name: "Flexible".into(),
            price_per_night: Decimal::new(25099, 2),
            refundable: true,
        },
    )
    .await?;
    assert_eq!(plan.room_type_id, other.room_type.id);
    assert_eq!(plan.name, "Flexible");
    assert_eq!(plan.price_per_night, Decimal::new(25099, 2));
    assert!(plan.refundable);
    assert!(
        pricing::repository::get_price_plans_by_room_type(&pool, f.room_type.id, 10, 0)
            .await?
            .is_empty()
    );
    let scoped =
        pricing::repository::get_price_plans_by_room_type(&pool, other.room_type.id, 10, 0).await?;
    assert_eq!(scoped.len(), 2);
    assert_eq!(
        pricing::repository::get_price_plans_by_room_type(&pool, other.room_type.id, 1, 1).await?
            [0]
        .id,
        scoped[1].id
    );
    let all = pricing::repository::get_all_price_plans(&pool, 10, 0).await?;
    assert_eq!(all.len(), 2);
    assert_eq!(
        pricing::repository::get_all_price_plans(&pool, 1, 1).await?[0].id,
        all[1].id
    );

    assert_eq!(
        rooms::repository::delete_room(&pool, f.room.id).await?.id,
        f.room.id
    );
    assert_eq!(
        pricing::repository::delete_price_plan(&pool, f.plan.id)
            .await?
            .id,
        f.plan.id
    );
    assert_eq!(
        rooms::repository::delete_room_type(&pool, f.room_type.id)
            .await?
            .id,
        f.room_type.id
    );
    assert_eq!(
        hotel::repository::delete_hotel(&pool, other.hotel.id)
            .await?
            .id,
        other.hotel.id
    );
    assert!(matches!(
        rooms::repository::get_room(&pool, other.room.id).await,
        Err(sqlx::Error::RowNotFound)
    ));
    assert!(matches!(
        rooms::repository::get_room_type(&pool, other.room_type.id).await,
        Err(sqlx::Error::RowNotFound)
    ));
    assert!(matches!(
        pricing::repository::get_price_plan(&pool, other.plan.id).await,
        Err(sqlx::Error::RowNotFound)
    ));
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn bookings_crud_status_and_scoped_lists(pool: PgPool) -> Result<(), sqlx::Error> {
    let f = Fixture::create(&pool).await?;
    let other = Fixture::create(&pool).await?;
    let booking = bookings::repository::create_booking(&pool, f.booking()).await?;
    assert_eq!(booking.status, Status::Pending);
    assert_eq!(booking.total_price, Decimal::new(24690, 2));
    assert_eq!(
        bookings::repository::get_booking(&pool, booking.id)
            .await?
            .check_in,
        f.booking().check_in
    );
    for status in [Status::Confirmed, Status::Cancelled, Status::Pending] {
        let updated =
            bookings::repository::update_booking_status(&pool, booking.id, status).await?;
        assert_eq!(updated.status, status);
        assert_eq!(updated.total_price, booking.total_price);
        assert_eq!(updated.created_at, booking.created_at);
        assert_eq!(
            bookings::repository::get_booking(&pool, booking.id)
                .await?
                .status,
            status
        );
    }
    let new = other.booking();
    let updated = bookings::repository::update_booking(
        &pool,
        booking.id,
        UpdateBooking {
            user_id: new.user_id,
            room_id: new.room_id,
            price_plan_id: new.price_plan_id,
            check_in: new.check_in,
            check_out: NaiveDate::from_ymd_opt(2026, 10, 13).unwrap(),
            guests: 1,
            status: Status::Confirmed,
            total_price: Decimal::new(37035, 2),
        },
    )
    .await?;
    assert_eq!(updated.user_id, other.user.id);
    assert_eq!(updated.room_id, other.room.id);
    assert_eq!(updated.price_plan_id, other.plan.id);
    assert_eq!(
        updated.check_out,
        NaiveDate::from_ymd_opt(2026, 10, 13).unwrap()
    );
    assert_eq!(updated.guests, 1);
    assert_eq!(updated.status, Status::Confirmed);
    assert_eq!(updated.total_price, Decimal::new(37035, 2));
    assert_eq!(updated.created_at, booking.created_at);
    let mut second = other.booking();
    second.check_in = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
    second.check_out = NaiveDate::from_ymd_opt(2026, 11, 12).unwrap();
    bookings::repository::create_booking(&pool, second).await?;
    assert!(
        bookings::repository::get_bookings_by_user(&pool, f.user.id, 10, 0)
            .await?
            .is_empty()
    );
    assert!(
        bookings::repository::get_bookings_by_room(&pool, f.room.id, 10, 0)
            .await?
            .is_empty()
    );
    let all = bookings::repository::get_all_bookings(&pool, 10, 0).await?;
    assert_eq!(all.len(), 2);
    assert_eq!(
        bookings::repository::get_all_bookings(&pool, 1, 1).await?[0].id,
        all[1].id
    );
    assert_eq!(
        bookings::repository::get_bookings_by_user(&pool, other.user.id, 10, 0)
            .await?
            .len(),
        2
    );
    assert_eq!(
        bookings::repository::get_bookings_by_user(&pool, other.user.id, 1, 1).await?[0].id,
        all[1].id
    );
    assert_eq!(
        bookings::repository::get_bookings_by_room(&pool, other.room.id, 10, 0)
            .await?
            .len(),
        2
    );
    assert_eq!(
        bookings::repository::get_bookings_by_room(&pool, other.room.id, 1, 1).await?[0].id,
        all[1].id
    );
    assert_eq!(
        bookings::repository::delete_booking(&pool, booking.id)
            .await?
            .id,
        booking.id
    );
    assert!(matches!(
        bookings::repository::get_booking(&pool, booking.id).await,
        Err(sqlx::Error::RowNotFound)
    ));
    assert!(matches!(
        bookings::repository::delete_booking(&pool, booking.id).await,
        Err(sqlx::Error::RowNotFound)
    ));
    assert!(matches!(
        bookings::repository::update_booking_status(&pool, booking.id, Status::Cancelled).await,
        Err(sqlx::Error::RowNotFound)
    ));
    Ok(())
}

#[sqlx::test(migrations = "./migrations")]
async fn database_constraints_are_preserved(pool: PgPool) -> Result<(), sqlx::Error> {
    let f = Fixture::create(&pool).await?;
    let mut invalid = f.booking();
    invalid.check_out = invalid.check_in;
    assert_database_code(
        bookings::repository::create_booking(&pool, invalid)
            .await
            .unwrap_err(),
        "23514",
    );
    let mut invalid = f.booking();
    invalid.guests = 0;
    assert_database_code(
        bookings::repository::create_booking(&pool, invalid)
            .await
            .unwrap_err(),
        "23514",
    );
    let mut invalid = f.booking();
    invalid.total_price = Decimal::new(-1, 2);
    assert_database_code(
        bookings::repository::create_booking(&pool, invalid)
            .await
            .unwrap_err(),
        "23514",
    );
    let mut invalid = f.booking();
    invalid.user_id = Uuid::new_v4();
    assert_database_code(
        bookings::repository::create_booking(&pool, invalid)
            .await
            .unwrap_err(),
        "23503",
    );
    assert_database_code(
        rooms::repository::create_room(
            &pool,
            CreateRoom {
                room_type_id: f.room_type.id,
                number: "101".into(),
            },
        )
        .await
        .unwrap_err(),
        "23505",
    );
    assert_database_code(
        rooms::repository::create_room_type(
            &pool,
            CreateRoomType {
                hotel_id: f.hotel.id,
                name: "Invalid".into(),
                description: None,
                capacity: 0,
                base_price: Decimal::ZERO,
            },
        )
        .await
        .unwrap_err(),
        "23514",
    );
    assert_database_code(
        pricing::repository::create_price_plan(
            &pool,
            CreatePricePlan {
                room_type_id: f.room_type.id,
                name: "Invalid".into(),
                price_per_night: Decimal::new(-1, 2),
                refundable: false,
            },
        )
        .await
        .unwrap_err(),
        "23514",
    );
    bookings::repository::create_booking(&pool, f.booking()).await?;
    assert_database_code(
        users::repository::delete_user(&pool, f.user.id)
            .await
            .unwrap_err(),
        "23503",
    );
    assert_database_code(
        rooms::repository::delete_room(&pool, f.room.id)
            .await
            .unwrap_err(),
        "23503",
    );
    assert_database_code(
        pricing::repository::delete_price_plan(&pool, f.plan.id)
            .await
            .unwrap_err(),
        "23503",
    );
    assert_database_code(
        hotel::repository::delete_hotel(&pool, f.hotel.id)
            .await
            .unwrap_err(),
        "23503",
    );
    assert!(
        hotel::repository::get_hotel(&pool, f.hotel.id)
            .await
            .is_ok()
    );
    assert!(rooms::repository::get_room(&pool, f.room.id).await.is_ok());
    Ok(())
}
