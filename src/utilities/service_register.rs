use crate::user::user_client::UserClient;
use common::errors::{ServiceError, ServiceResult};
use std::sync::Arc;
use tracing::info;

use super::config::AppConfig;
use super::states::token_service::StateTokenService;
use super::states::user_service::StateUserService;
use super::token::JwtService;

#[derive(Clone)]
pub struct ServiceRegister {
    pub user_service: StateUserService,
    pub token_service: StateTokenService,
}

impl ServiceRegister {
    pub async fn new(config: Arc<AppConfig>) -> ServiceResult<Self> {
        info!("parsing config for addresses...");
        let user_service_address = format!("{}:{}", &config.user_host, &config.user_port);

        info!("initializing utility services...");
        let token_service = JwtService::new(config);

        info!("utility services initialized, building feature services...");
        let user_service = UserClient::connect(user_service_address)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(String::from(
                    "Unable to initialize user microservice",
                ))
            })?;

        info!("features services successfully initialized!");
        Ok(ServiceRegister {
            user_service: StateUserService::new(user_service),
            token_service: StateTokenService::new(token_service),
        })
    }
}
