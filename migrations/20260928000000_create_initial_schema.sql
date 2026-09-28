CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    role TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT users_email_not_blank CHECK (BTRIM(email) <> ''),
    CONSTRAINT users_password_hash_not_blank CHECK (BTRIM(password_hash) <> ''),
    CONSTRAINT users_first_name_not_blank CHECK (BTRIM(first_name) <> ''),
    CONSTRAINT users_last_name_not_blank CHECK (BTRIM(last_name) <> ''),
    CONSTRAINT users_role_valid CHECK (role IN ('customer', 'manager', 'admin'))
);

CREATE UNIQUE INDEX users_email_unique_ci ON users (LOWER(email));

CREATE TABLE hotels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    city TEXT NOT NULL,
    address TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT hotels_name_not_blank CHECK (BTRIM(name) <> ''),
    CONSTRAINT hotels_city_not_blank CHECK (BTRIM(city) <> ''),
    CONSTRAINT hotels_address_not_blank CHECK (BTRIM(address) <> '')
);

CREATE INDEX hotels_city_idx ON hotels (LOWER(city));

CREATE TABLE room_types (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hotel_id UUID NOT NULL REFERENCES hotels(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    capacity INTEGER NOT NULL,
    base_price NUMERIC(12, 2) NOT NULL,

    CONSTRAINT room_types_name_not_blank CHECK (BTRIM(name) <> ''),
    CONSTRAINT room_types_capacity_positive CHECK (capacity > 0),
    CONSTRAINT room_types_base_price_non_negative CHECK (base_price >= 0),
    CONSTRAINT room_types_hotel_name_unique UNIQUE (hotel_id, name)
);

CREATE INDEX room_types_hotel_id_idx ON room_types (hotel_id);

CREATE TABLE rooms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_type_id UUID NOT NULL REFERENCES room_types(id) ON DELETE CASCADE,
    number TEXT NOT NULL,

    CONSTRAINT rooms_number_not_blank CHECK (BTRIM(number) <> ''),
    CONSTRAINT rooms_type_number_unique UNIQUE (room_type_id, number)
);

CREATE INDEX rooms_room_type_id_idx ON rooms (room_type_id);

CREATE TABLE price_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    room_type_id UUID NOT NULL REFERENCES room_types(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    price_per_night NUMERIC(12, 2) NOT NULL,
    refundable BOOLEAN NOT NULL DEFAULT FALSE,

    CONSTRAINT price_plans_name_not_blank CHECK (BTRIM(name) <> ''),
    CONSTRAINT price_plans_price_non_negative CHECK (price_per_night >= 0),
    CONSTRAINT price_plans_room_type_name_unique UNIQUE (room_type_id, name)
);

CREATE INDEX price_plans_room_type_id_idx ON price_plans (room_type_id);

CREATE TABLE bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE RESTRICT,
    price_plan_id UUID NOT NULL REFERENCES price_plans(id) ON DELETE RESTRICT,
    check_in DATE NOT NULL,
    check_out DATE NOT NULL,
    guests INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    total_price NUMERIC(12, 2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT bookings_dates_valid CHECK (check_out > check_in),
    CONSTRAINT bookings_guests_positive CHECK (guests > 0),
    CONSTRAINT bookings_status_valid CHECK (status IN ('pending', 'confirmed', 'cancelled')),
    CONSTRAINT bookings_total_price_non_negative CHECK (total_price >= 0)
);

CREATE INDEX bookings_user_id_idx ON bookings (user_id);
CREATE INDEX bookings_room_id_idx ON bookings (room_id);
CREATE INDEX bookings_price_plan_id_idx ON bookings (price_plan_id);
CREATE INDEX bookings_room_dates_idx ON bookings (room_id, check_in, check_out)
    WHERE status IN ('pending', 'confirmed');
