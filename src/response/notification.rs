use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct NotificationEndpointResponse {
    pub message: String,
}
