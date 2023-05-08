use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default)]
pub struct RegisterEndpointRequest {
    #[validate(required, length(min = 6, max = 30))]
    pub username: Option<String>,
    #[validate(required, length(min = 1), email(message = "Email is invalid"))]
    pub email: Option<String>,
    #[validate(required, length(min = 8, max = 30))]
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct LoginEndpointRequest {
    #[validate(required, length(min = 1), email(message = "Email is invalid"))]
    pub email: Option<String>,
    #[validate(required, length(min = 8, max = 30))]
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct RefreshtokenEndpointRequest {
    #[validate(required, length(min = 1))]
    pub token: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Validate)]
pub struct UpdateEndpointRequest {
    #[validate(email(message = "Email is invalid"))]
    pub email: Option<String>,
    #[validate(length(min = 6, max = 30))]
    pub username: Option<String>,
    #[validate(length(min = 8))]
    pub password: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}
