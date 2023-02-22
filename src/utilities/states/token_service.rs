use std::ops::{Deref, DerefMut};

use axum::extract::FromRef;

use crate::utilities::{service_register::ServiceRegister, token::JwtService};

#[derive(Clone)]
pub struct StateTokenService(pub JwtService);

impl FromRef<ServiceRegister> for StateTokenService {
    fn from_ref(input: &ServiceRegister) -> Self {
        input.token_service.clone()
    }
}

impl StateTokenService {
    pub fn new(token_service: JwtService) -> Self {
        Self(token_service)
    }
}

impl Deref for StateTokenService {
    type Target = JwtService;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StateTokenService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
