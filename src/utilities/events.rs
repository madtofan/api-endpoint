use std::{fmt, str::FromStr};

use madtofan_microservice_common::{
    errors::{ServiceError, ServiceResult},
    notification::MessageResponse,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ChannelTag {
    UserId(i64),
    ChannelId(String),
    Broadcast,
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "_type")]
pub enum EventMessage {
    User(NotificationMessage),
    Channel(NotificationMessage),
    Broadcast(NotificationMessage),
}

impl FromStr for ChannelTag {
    type Err = ServiceError;

    fn from_str(input: &str) -> ServiceResult<Self> {
        let split = input.split(':').collect::<Vec<&str>>();
        if split.is_empty() {
            return Err(ServiceError::BadRequest(
                "target cannot be parsed".to_string(),
            ));
        }
        match *split.first().unwrap() {
            "User" => {
                let user_id: i64 = split
                    .get(1)
                    .unwrap()
                    .parse()
                    .map_err(|_| ServiceError::BadRequest("invalid user id".to_string()))?;

                Ok(ChannelTag::UserId(user_id))
            }
            "Channel" => Ok(ChannelTag::ChannelId(split.get(1).unwrap().to_string())),
            "Broadcast" => Ok(ChannelTag::Broadcast),
            _ => Err(ServiceError::BadRequest("invalid channel name".to_string())),
        }
    }
}

impl fmt::Display for ChannelTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChannelTag::UserId(user) => write!(f, "User:{}", user),
            ChannelTag::ChannelId(channel) => write!(f, "Channel:{}", channel),
            ChannelTag::Broadcast => write!(f, "Broadcast"),
        }
    }
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/notification/")]
pub struct NotificationMessage {
    pub id: i64,
    pub datetime: i64,
    pub channel: String,
    pub subject: String,
    pub message: String,
}

impl NotificationMessage {
    pub fn from_message_response(message_response: MessageResponse) -> Self {
        Self {
            id: message_response.id,
            datetime: message_response.date,
            channel: message_response.channel,
            subject: message_response.subject,
            message: message_response.message,
        }
    }
}
