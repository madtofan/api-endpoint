use madtofan_microservice_common::{
    email::email_client::EmailClient,
    errors::{ServiceError, ServiceResult},
    templating::templating_client::TemplatingClient,
    user::user_client::UserClient,
};
use std::sync::Arc;
use tracing::info;

use super::config::AppConfig;
use super::states::email_service::StateEmailService;
use super::states::templating_service::StateTemplatingService;
use super::states::token_service::StateTokenService;
use super::states::user_service::StateUserService;
use super::token::JwtService;

#[derive(Clone)]
pub struct ServiceRegister {
    pub user_service: StateUserService,
    pub email_service: StateEmailService,
    pub templating_service: StateTemplatingService,
    pub token_service: StateTokenService,
}

impl ServiceRegister {
    pub async fn new(config: Arc<AppConfig>) -> ServiceResult<Self> {
        info!("parsing config for addresses...");
        let user_service_address = format!("{}:{}", &config.user_host, &config.user_port);
        let email_service_address = format!("{}:{}", &config.email_host, &config.email_port);
        let templating_service_address =
            format!("{}:{}", &config.templating_host, &config.templating_port);

        info!("initializing utility services...");
        let token_service = JwtService::new(config);

        info!("utility services initialized, building feature services...");
        info!("user addr: {:#?}", user_service_address);
        info!("email addr: {:#?}", email_service_address);
        info!("templating addr: {:#?}", templating_service_address);
        let user_service = UserClient::connect(user_service_address)
            .await
            .map_err(|err| {
                ServiceError::InternalServerErrorWithContext(format!(
                    "Unable to initialize user microservice: {:#?}",
                    err.to_string(),
                ))
            })?;
        let email_service = EmailClient::connect(email_service_address)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(String::from(
                    "Unable to initialize email microservice",
                ))
            })?;
        let templating_service = TemplatingClient::connect(templating_service_address)
            .await
            .map_err(|_| {
                ServiceError::InternalServerErrorWithContext(String::from(
                    "Unable to initialize templating microservice",
                ))
            })?;

        info!("features services successfully initialized!");
        Ok(ServiceRegister {
            user_service: StateUserService::new(user_service),
            email_service: StateEmailService::new(email_service),
            templating_service: StateTemplatingService::new(templating_service),
            token_service: StateTokenService::new(token_service),
        })
    }
}
