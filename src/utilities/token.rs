use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use madtofan_microservice_common::errors::{ServiceError, ServiceResult};
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};
use time::OffsetDateTime;

use super::config::AppConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct BearerClaims {
    pub sub: String,
    pub user_id: i64,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshClaims {
    user_id: i64,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyRegistrationClaim {
    user_id: i64,
    exp: usize,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Tokens {
    pub bearer: String,
    pub refresh: String,
}

#[derive(Clone)]
pub struct JwtService {
    config: Arc<AppConfig>,
}

impl JwtService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    pub fn create_token(&self, user_id: i64, email: &str) -> ServiceResult<Tokens> {
        let from_now = Duration::from_secs(900);
        let expired_future_time = SystemTime::now().add(from_now);
        let exp = OffsetDateTime::from(expired_future_time);

        let bearer_claims = BearerClaims {
            sub: String::from(email),
            exp: exp.unix_timestamp() as usize,
            user_id,
        };

        let bearer = encode(
            &Header::default(),
            &bearer_claims,
            &EncodingKey::from_secret(self.config.bearer_secret.as_bytes()),
        )
        .map_err(|err| ServiceError::InternalServerErrorWithContext(err.to_string()))?;

        let from_now = Duration::from_secs(604800);
        let expired_future_time = SystemTime::now().add(from_now);
        let exp = OffsetDateTime::from(expired_future_time);

        let refresh_claims = RefreshClaims {
            exp: exp.unix_timestamp() as usize,
            user_id,
        };

        let refresh = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.config.refresh_secret.as_bytes()),
        )
        .map_err(|err| ServiceError::InternalServerErrorWithContext(err.to_string()))?;

        Ok(Tokens { bearer, refresh })
    }

    pub fn decode_bearer_token(&self, token: &str) -> ServiceResult<BearerClaims> {
        let decoded_token = decode::<BearerClaims>(
            token,
            &DecodingKey::from_secret(self.config.bearer_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ServiceError::Unauthorized)?;

        let now = OffsetDateTime::from(SystemTime::now()).unix_timestamp() as usize;
        if decoded_token.claims.exp < now {
            return Err(ServiceError::Unauthorized);
        }

        Ok(decoded_token.claims)
    }

    pub fn refresh_tokens(&self, refresh: &str, bearer: &str) -> ServiceResult<Tokens> {
        let decoded_refresh_token = decode::<RefreshClaims>(
            refresh,
            &DecodingKey::from_secret(self.config.refresh_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ServiceError::Unauthorized)?;

        let now = OffsetDateTime::from(SystemTime::now()).unix_timestamp() as usize;
        if decoded_refresh_token.claims.exp < now {
            return Err(ServiceError::Unauthorized);
        }

        let decoded_bearer_token = decode::<BearerClaims>(
            bearer,
            &DecodingKey::from_secret(self.config.bearer_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ServiceError::Unauthorized)?;

        Ok(self.create_token(
            decoded_bearer_token.claims.user_id,
            &decoded_bearer_token.claims.sub,
        )?)
    }

    pub fn create_verify_registration_token(&self, user_id: i64) -> ServiceResult<String> {
        let from_now = Duration::from_secs(86400);
        let expired_future_time = SystemTime::now().add(from_now);
        let exp = OffsetDateTime::from(expired_future_time);

        let bearer_claims = VerifyRegistrationClaim {
            exp: exp.unix_timestamp() as usize,
            user_id,
        };

        let bearer = encode(
            &Header::default(),
            &bearer_claims,
            &EncodingKey::from_secret(self.config.verify_registration_secret.as_bytes()),
        )
        .map_err(|err| ServiceError::InternalServerErrorWithContext(err.to_string()))?;

        Ok(bearer)
    }

    pub fn extract_verify_registration_token(&self, token: &str) -> ServiceResult<i64> {
        let decoded_token = decode::<VerifyRegistrationClaim>(
            token,
            &DecodingKey::from_secret(self.config.verify_registration_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ServiceError::Unauthorized)?;

        let now = OffsetDateTime::from(SystemTime::now()).unix_timestamp() as usize;
        if decoded_token.claims.exp < now {
            return Err(ServiceError::Unauthorized);
        }

        Ok(decoded_token.claims.user_id)
    }
}
