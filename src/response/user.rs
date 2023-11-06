use madtofan_microservice_common::user::{ListResponse, UserResponse};
use serde::{Deserialize, Serialize};

use crate::utilities::token::Tokens;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserEndpointResponse {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub roles: Vec<RolePermissions>,
}

impl UserEndpointResponse {
    pub fn from_user_response(user_response: UserResponse) -> Self {
        Self {
            id: user_response.id,
            email: user_response.email,
            first_name: user_response.first_name,
            last_name: user_response.last_name,
            bio: user_response.bio,
            image: user_response.image,
            roles: user_response
                .roles
                .into_iter()
                .map(|role| RolePermissions {
                    role_name: role.name,
                    permissions: role.permissions,
                })
                .collect::<Vec<RolePermissions>>(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ObtainTokenResponse {
    pub refresh_token: String,
    pub bearer_token: String,
}

impl ObtainTokenResponse {
    pub fn from_tokens(tokens: Tokens) -> Self {
        Self {
            refresh_token: tokens.refresh,
            bearer_token: tokens.bearer,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RegisterUserEndpointResponse {
    pub email: String,
    pub verify_token: String,
}

#[derive(Deserialize, Serialize)]
pub struct Roles {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct RolesListResponse {
    pub roles: Vec<Roles>,
    pub count: i64,
}

impl RolesListResponse {
    pub fn from_list_response(list_response: ListResponse) -> Self {
        Self {
            roles: list_response
                .list
                .into_iter()
                .map(|role| Roles {
                    id: role.id,
                    name: role.name,
                })
                .collect(),
            count: list_response.count,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Permissions {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct PermissionsListResponse {
    pub roles: Vec<Permissions>,
    pub count: i64,
}

impl PermissionsListResponse {
    pub fn from_list_response(list_response: ListResponse) -> Self {
        Self {
            roles: list_response
                .list
                .into_iter()
                .map(|role| Permissions {
                    id: role.id,
                    name: role.name,
                })
                .collect(),
            count: list_response.count,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RolePermissions {
    pub role_name: String,
    pub permissions: Vec<String>,
}
