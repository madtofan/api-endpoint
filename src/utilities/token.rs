use common::errors::{ServiceError, ServiceResult};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use time::OffsetDateTime;

use super::config::AppConfig;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    user_id: i64,
    exp: usize,
}

#[derive(Clone)]
pub struct JwtService {
    config: Arc<AppConfig>,
}

impl JwtService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    pub fn create_token(&self, user_id: i64, email: &str) -> ServiceResult<String> {
        let from_now = Duration::from_secs(3600);
        let expired_future_time = SystemTime::now().add(from_now);
        let exp = OffsetDateTime::from(expired_future_time);

        let claims = Claims {
            sub: String::from(email),
            exp: exp.unix_timestamp() as usize,
            user_id,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.token_secret.as_bytes()),
        )
        .map_err(|err| ServiceError::InternalServerErrorWithContext(err.to_string()))?;

        Ok(token)
    }

    pub fn get_user_id_from_token(&self, token: &str) -> ServiceResult<i64> {
        let decoded_token = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.token_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ServiceError::Unauthorized)?;

        Ok(decoded_token.claims.user_id)
    }
}
