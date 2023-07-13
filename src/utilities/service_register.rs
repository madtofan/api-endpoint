use madtofan_microservice_common::{
    email::email_client::EmailClient, errors::ServiceResult,
    templating::templating_client::TemplatingClient, user::user_client::UserClient,
};
use std::sync::Arc;
use tonic::transport::Endpoint;
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
        let user_service_address_string = format!("{}:{}", &config.user_host, &config.user_port);
        let user_service_address = Box::leak(user_service_address_string.into_boxed_str());
        let email_service_address_string = format!("{}:{}", &config.email_host, &config.email_port);
        let email_service_address = Box::leak(email_service_address_string.into_boxed_str());
        let templating_service_address_string =
            format!("{}:{}", &config.templating_host, &config.templating_port);
        let templating_service_address =
            Box::leak(templating_service_address_string.into_boxed_str());

        info!("initializing utility services...");
        let token_service = JwtService::new(config);

        info!("utility services initialized, building feature services...");
        let user_endpoint = Endpoint::from_static(user_service_address).connect_lazy();
        let user_service = UserClient::new(user_endpoint);
        let email_endpoint = Endpoint::from_static(email_service_address).connect_lazy();
        let email_service = EmailClient::new(email_endpoint);
        let templating_endpoint = Endpoint::from_static(templating_service_address).connect_lazy();
        let templating_service = TemplatingClient::new(templating_endpoint);

        info!("features services successfully initialized!");
        Ok(ServiceRegister {
            user_service: StateUserService::new(user_service),
            email_service: StateEmailService::new(email_service),
            templating_service: StateTemplatingService::new(templating_service),
            token_service: StateTokenService::new(token_service),
        })
    }
}
