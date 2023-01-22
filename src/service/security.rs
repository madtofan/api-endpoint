use crate::{
    data::config::AppConfig,
    domain::errors::{ServiceError, ServiceResult},
};
use argon2::Config;
use mockall::automock;
use std::sync::Arc;

#[automock]
pub trait SecurityServiceTrait {
    fn hash_password(&self, raw_password: &str) -> ServiceResult<String>;

    fn verify_password(
        &self,
        stored_password: &str,
        attempted_password: String,
    ) -> ServiceResult<bool>;
}

pub type DynSecurityServiceTrait = Arc<dyn SecurityServiceTrait + Send + Sync>;

pub struct SecurityService {
    config: Arc<AppConfig>,
}

impl SecurityService {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }
}

impl SecurityServiceTrait for SecurityService {
    fn hash_password(&self, raw_password: &str) -> ServiceResult<String> {
        let password_bytes = raw_password.as_bytes();
        let hashed_password = argon2::hash_encoded(
            password_bytes,
            self.config.argon_salt.as_bytes(),
            &Config::default(),
        )
        .unwrap();

        Ok(hashed_password)
    }

    fn verify_password(
        &self,
        stored_password: &str,
        attempted_password: String,
    ) -> ServiceResult<bool> {
        let hashes_match =
            argon2::verify_encoded(stored_password, attempted_password.as_bytes())
                .map_err(|err| ServiceError::InternalServerErrorWithContext(err.to_string()))?;

        Ok(hashes_match)
    }
}
