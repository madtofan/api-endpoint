use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Serialize, Deserialize, Debug, Validate, Default)]
pub struct RegisterEndpointRequest {
    #[validate(required, length(min = 1), email(message = "Email is invalid"))]
    pub email: Option<String>,
    #[validate(required, length(min = 8, max = 30))]
    pub password: Option<String>,
    #[validate(required)]
    pub first_name: Option<String>,
    #[validate(required)]
    pub last_name: Option<String>,
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
    #[validate(length(min = 8))]
    pub password: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Validate)]
pub struct AddRolePermissionRequest {
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Validate)]
pub struct AuthorizeRevokeUserRoleRequest {
    pub roles: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Default, Validate)]
pub struct AuthorizeRevokeRolePermissionRequest {
    pub permissions: Option<Vec<String>>,
}
