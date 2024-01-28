use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utilities::events::NotificationMessage;

#[derive(Serialize, Deserialize, Default, Debug, TS)]
#[ts(export, export_to = "bindings/notification/")]
pub struct NotificationEndpointResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "bindings/notification/")]
pub struct NotificationLogsEndpointResponse {
    pub notifications: Vec<NotificationMessage>,
    pub count: i64,
}
