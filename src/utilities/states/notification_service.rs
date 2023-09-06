use std::ops::{Deref, DerefMut};

use axum::extract::FromRef;
use madtofan_microservice_common::notification::notification_client::NotificationClient;
use tonic::transport::Channel;

use crate::utilities::service_register::ServiceRegister;

#[derive(Clone)]
pub struct StateNotificationService(pub NotificationClient<Channel>);

impl FromRef<ServiceRegister> for StateNotificationService {
    fn from_ref(input: &ServiceRegister) -> Self {
        input.notification_service.clone()
    }
}

impl StateNotificationService {
    pub fn new(notification_client: NotificationClient<Channel>) -> Self {
        Self(notification_client)
    }
}

impl Deref for StateNotificationService {
    type Target = NotificationClient<Channel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StateNotificationService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
