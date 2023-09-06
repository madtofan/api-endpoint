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
pub struct RefreshClaims {
    pub user_id: i64,
    pub user_email: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyRegistrationClaim {
    user_id: i64,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationSenderClaim {
    channel: String,
    email: String,
    exp: usize,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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
        let from_now = Duration::from_secs(60);
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
            user_email: String::from(email),
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

        Ok(decoded_token.claims)
    }

    pub fn decode_refresh_token(&self, refresh: &str) -> ServiceResult<RefreshClaims> {
        let decoded_token = decode::<RefreshClaims>(
            refresh,
            &DecodingKey::from_secret(self.config.refresh_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ServiceError::Unauthorized)?;

        Ok(decoded_token.claims)
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

    pub fn decode_verify_registration_token(&self, token: &str) -> ServiceResult<i64> {
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

    pub fn create_notification_sender_token(
        &self,
        channel: &str,
        email: &str,
    ) -> ServiceResult<String> {
        let one_year_from_now = Duration::from_secs(31622400);
        let expired_future_time = SystemTime::now().add(one_year_from_now);
        let exp = OffsetDateTime::from(expired_future_time);

        let bearer_claims = NotificationSenderClaim {
            email: email.to_string(),
            channel: channel.to_string(),
            exp: exp.unix_timestamp() as usize,
        };

        let bearer = encode(
            &Header::default(),
            &bearer_claims,
            &EncodingKey::from_secret(self.config.verify_registration_secret.as_bytes()),
        )
        .map_err(|err| ServiceError::InternalServerErrorWithContext(err.to_string()))?;

        Ok(bearer)
    }

    pub fn decode_notification_sender_token(
        &self,
        token: &str,
    ) -> ServiceResult<NotificationSenderClaim> {
        let decoded_token = decode::<NotificationSenderClaim>(
            token,
            &DecodingKey::from_secret(self.config.verify_registration_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| ServiceError::Unauthorized)?;

        let now = OffsetDateTime::from(SystemTime::now()).unix_timestamp() as usize;
        if decoded_token.claims.exp < now {
            return Err(ServiceError::Unauthorized);
        }

        Ok(decoded_token.claims)
    }
}
