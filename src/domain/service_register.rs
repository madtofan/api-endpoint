use crate::{
    data::{
        config::AppConfig,
        connection_pool::ServiceConnectionPool,
        repository::{DynUserRepositoryTrait, UserRepository},
    },
    service::{
        security::{DynSecurityServiceTrait, SecurityService},
        token::{DynTokenServiceTrait, JwtService},
        user::{DynUserServiceTrait, UserService},
    },
};
use std::sync::Arc;
use tracing::info;

#[derive(Clone)]
pub struct ServiceRegister {
    pub user_service: DynUserServiceTrait,
    pub token_service: DynTokenServiceTrait,
}

impl ServiceRegister {
    pub fn new(pool: ServiceConnectionPool, config: Arc<AppConfig>) -> Self {
        info!("initializing utility services...");
        let security_service =
            Arc::new(SecurityService::new(config.clone())) as DynSecurityServiceTrait;
        let token_service = Arc::new(JwtService::new(config)) as DynTokenServiceTrait;

        info!("utility services initialized, building feature services...");
        let users_repository =
            Arc::new(UserRepository::new(pool.clone())) as DynUserRepositoryTrait;
        let user_service = Arc::new(UserService::new(
            users_repository.clone(),
            security_service,
            token_service.clone(),
        )) as DynUserServiceTrait;

        info!("features services successfully initialized!");

        ServiceRegister {
            user_service,
            token_service,
        }
    }
}
