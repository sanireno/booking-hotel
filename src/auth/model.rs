use crate::users::model::UserRole;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: UserRole,
    pub exp: u64,
    pub iat: u64,
    pub nbf: u64,
    pub iss: String,
    pub aud: String,
}
