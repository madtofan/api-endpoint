use std::ops::{Deref, DerefMut};

use axum::extract::FromRef;
use tonic::transport::Channel;

use crate::{
    templating::templating_client::TemplatingClient, utilities::service_register::ServiceRegister,
};

#[derive(Clone)]
pub struct StateTemplatingService(pub TemplatingClient<Channel>);

impl FromRef<ServiceRegister> for StateTemplatingService {
    fn from_ref(input: &ServiceRegister) -> Self {
        input.templating_service.clone()
    }
}

impl StateTemplatingService {
    pub fn new(templating_client: TemplatingClient<Channel>) -> Self {
        Self(templating_client)
    }
}

impl Deref for StateTemplatingService {
    type Target = TemplatingClient<Channel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StateTemplatingService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
