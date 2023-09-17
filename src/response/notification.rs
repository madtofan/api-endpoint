use serde::{Deserialize, Serialize};

use crate::utilities::events::NotificationMessage;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct NotificationEndpointResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct NotificationLogsEndpointResponse {
    pub notifications: Vec<NotificationMessage>,
    pub count: i64,
}
