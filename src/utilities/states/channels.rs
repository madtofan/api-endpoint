use std::ops::{Deref, DerefMut};

use axum::extract::FromRef;
use tagged_channels::TaggedChannels;

use crate::utilities::{
    events::{ChannelTag, EventMessage},
    service_register::ServiceRegister,
};

#[derive(Clone)]
pub struct StateChannelsService(pub TaggedChannels<EventMessage, ChannelTag>);

impl FromRef<ServiceRegister> for StateChannelsService {
    fn from_ref(input: &ServiceRegister) -> Self {
        input.channel_service.clone()
    }
}

impl StateChannelsService {
    pub fn new(channel_service: TaggedChannels<EventMessage, ChannelTag>) -> Self {
        Self(channel_service)
    }
}

impl Deref for StateChannelsService {
    type Target = TaggedChannels<EventMessage, ChannelTag>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StateChannelsService {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
