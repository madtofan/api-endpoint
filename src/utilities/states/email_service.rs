use std::ops::{Deref, DerefMut};

use axum::extract::FromRef;
use madtofan_microservice_common::email::email_client::EmailClient;
use tonic::transport::Channel;

use crate::utilities::service_register::ServiceRegister;

#[derive(Clone)]
pub struct StateEmailService(pub EmailClient<Channel>);

impl FromRef<ServiceRegister> for StateEmailService {
    fn from_ref(input: &ServiceRegister) -> Self {
        input.email_service.clone()
    }
}

impl StateEmailService {
    pub fn new(email_client: EmailClient<Channel>) -> Self {
        Self(email_client)
    }
}

impl Deref for StateEmailService {
    type Target = EmailClient<Channel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StateEmailService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
