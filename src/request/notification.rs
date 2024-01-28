use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default, TS)]
#[ts(export, export_to = "bindings/notification/")]
pub struct SendNotificationEndpointRequest {
    #[validate(required)]
    pub address: Option<String>,
    #[validate(required, length(min = 6, max = 30))]
    pub subject: Option<String>,
    #[validate(required)]
    pub message: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default, TS)]
#[ts(export, export_to = "bindings/notification/")]
pub struct AddGroupEndpointRequest {
    #[validate(required, length(min = 6, max = 30))]
    pub group_name: Option<String>,
    #[validate(required, length(min = 1), email(message = "Email is invalid"))]
    pub admin_email: Option<String>,
}
