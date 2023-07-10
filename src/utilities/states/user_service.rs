use std::ops::{Deref, DerefMut};

use axum::extract::FromRef;
use madtofan_microservice_common::user::user_client::UserClient;
use tonic::transport::Channel;

use crate::utilities::service_register::ServiceRegister;

#[derive(Clone)]
pub struct StateUserService(pub UserClient<Channel>);

impl FromRef<ServiceRegister> for StateUserService {
    fn from_ref(input: &ServiceRegister) -> Self {
        input.user_service.clone()
    }
}

impl StateUserService {
    pub fn new(user_client: UserClient<Channel>) -> Self {
        Self(user_client)
    }
}

impl Deref for StateUserService {
    type Target = UserClient<Channel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StateUserService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
