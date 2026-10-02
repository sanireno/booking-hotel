use crate::{
    app_error::AppError,
    auth::model::Claims,
    config::{AuthConfig, ConfigError},
    users::model::User,
};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, get_current_timestamp,
};

const ISSUER: &str = "booking-hotel";
const AUDIENCE: &str = "booking-hotel-api";

#[derive(Clone)]
pub struct TokenService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    ttl_seconds: u64,
}

impl TokenService {
    pub fn new(config: &AuthConfig) -> Result<Self, ConfigError> {
        config.validate()?;
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        validation.validate_nbf = true;
        validation.set_required_spec_claims(&["sub", "exp", "nbf", "iss", "aud"]);
        validation.set_issuer(&[ISSUER]);
        validation.set_audience(&[AUDIENCE]);
        Ok(Self {
            encoding_key: EncodingKey::from_secret(config.jwt_secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            validation,
            ttl_seconds: config.jwt_ttl_seconds,
        })
    }

    pub fn ttl_seconds(&self) -> u64 {
        self.ttl_seconds
    }

    pub fn issue(&self, user: &User) -> Result<String, AppError> {
        let now = get_current_timestamp();
        let claims = Claims {
            sub: user.id,
            role: user.role,
            exp: now
                .checked_add(self.ttl_seconds)
                .ok_or(AppError::InternalServerError)?,
            iat: now,
            nbf: now,
            iss: ISSUER.into(),
            aud: AUDIENCE.into(),
        };
        encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key).map_err(|error| {
            tracing::error!(%error, "access token generation failed");
            AppError::InternalServerError
        })
    }

    pub fn verify(&self, token: &str) -> Result<Claims, AppError> {
        let claims = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|_| AppError::Unauthorized)?
            .claims;
        if claims.exp <= get_current_timestamp() {
            return Err(AppError::Unauthorized);
        }
        Ok(claims)
    }
}
