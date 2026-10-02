use crate::{
    app_error::AppError,
    app_state::AppState,
    auth::dto::{AuthResponse, LoginRequest, RegisterRequest, UserResponse},
    users::{
        model::{CreateUser, User, UserRole},
        repository,
    },
};
use email_address::{EmailAddress, Options};

pub async fn register(
    state: &AppState,
    request: RegisterRequest,
) -> Result<AuthResponse, AppError> {
    let email = normalize_email(&request.email)?;
    let first_name = validate_name(&request.first_name, "first_name")?;
    let last_name = validate_name(&request.last_name, "last_name")?;
    if !(12..=128).contains(&request.password.chars().count())
        || request.password.len() > 512
        || request.password.trim().is_empty()
    {
        return Err(AppError::Validation(
            "Password must contain 12 to 128 characters".into(),
        ));
    }
    let password_hash = state.passwords.hash(request.password).await?;
    let user = repository::create_user(
        &state.pool,
        CreateUser {
            email,
            password_hash,
            first_name,
            last_name,
            role: UserRole::Customer,
        },
    )
    .await?;
    auth_response(state, user)
}

pub async fn login(state: &AppState, request: LoginRequest) -> Result<AuthResponse, AppError> {
    let email = normalize_email(&request.email)?;
    if request.password.is_empty() || request.password.len() > 512 {
        return Err(AppError::Unauthorized);
    }
    let user = match repository::get_user_by_email(&state.pool, &email).await {
        Ok(user) => Some(user),
        Err(sqlx::Error::RowNotFound) => None,
        Err(error) => return Err(error.into()),
    };
    // Unknown accounts still perform password verification to reduce timing differences.
    let valid = state
        .passwords
        .verify(
            request.password,
            user.as_ref().map(|user| user.password_hash.clone()),
        )
        .await?;
    let user = user.filter(|_| valid).ok_or(AppError::Unauthorized)?;
    auth_response(state, user)
}

pub async fn authenticate(state: &AppState, token: &str) -> Result<UserResponse, AppError> {
    let claims = state.tokens.verify(token)?;
    let user = match repository::get_user(&state.pool, claims.sub).await {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => return Err(AppError::Unauthorized),
        Err(error) => return Err(error.into()),
    };
    // Authorization uses the current database role, rather than a stale JWT role.
    Ok(user.into())
}

fn auth_response(state: &AppState, user: User) -> Result<AuthResponse, AppError> {
    Ok(AuthResponse {
        access_token: state.tokens.issue(&user)?,
        token_type: "Bearer",
        expires_in: state.tokens.ttl_seconds(),
        user: user.into(),
    })
}

fn normalize_email(email: &str) -> Result<String, AppError> {
    let email = email.trim();
    if email.len() > 254
        || EmailAddress::parse_with_options(email, Options::default().without_display_text())
            .is_err()
    {
        return Err(AppError::Validation("Invalid email address".into()));
    }
    Ok(email.to_ascii_lowercase())
}

fn validate_name(name: &str, field: &str) -> Result<String, AppError> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
        return Err(AppError::Validation(format!(
            "{field} must contain 1 to 100 characters without control characters"
        )));
    }
    Ok(name.into())
}
