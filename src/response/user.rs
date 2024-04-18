use madtofan_microservice_common::user::{ListResponse, UserList, UserListResponse, UserResponse};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::utilities::token::Tokens;

#[derive(Serialize, Deserialize, Default, Debug, TS)]
#[ts(export, export_to = "bindings/user/")]
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

#[derive(Serialize, Deserialize, Default, Debug, TS)]
#[ts(export, export_to = "bindings/user/")]
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

#[derive(Serialize, Deserialize, Default, Debug, TS)]
#[ts(export, export_to = "bindings/user/")]
pub struct RegisterUserEndpointResponse {
    pub email: String,
    pub verify_token: String,
}

#[derive(Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/user/")]
pub struct Roles {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "bindings/user/")]
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

#[derive(Deserialize, Serialize, TS)]
#[ts(export, export_to = "bindings/user/")]
pub struct Permissions {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "bindings/user/")]
pub struct PermissionsListResponse {
    pub permissions: Vec<Permissions>,
    pub count: i64,
}

impl PermissionsListResponse {
    pub fn from_list_response(list_response: ListResponse) -> Self {
        Self {
            permissions: list_response
                .list
                .into_iter()
                .map(|permission| Permissions {
                    id: permission.id,
                    name: permission.name,
                })
                .collect(),
            count: list_response.count,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug, TS)]
#[ts(export, export_to = "bindings/user/")]
pub struct RolePermissions {
    pub role_name: String,
    pub permissions: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "bindings/user/")]
pub struct UserListEndpoint {
    pub id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl From<UserList> for UserListEndpoint {
    fn from(user_list: UserList) -> Self {
        Self {
            id: user_list.id,
            email: user_list.email,
            first_name: user_list.first_name,
            last_name: user_list.last_name,
            bio: user_list.bio,
            image: user_list.image,
        }
    }
}

#[derive(Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "bindings/user/")]
pub struct UserListEndpointResponse {
    pub users: Vec<UserListEndpoint>,
    pub count: i64,
}

impl UserListEndpointResponse {
    pub fn from_list_response(list_response: UserListResponse) -> Self {
        Self {
            users: list_response
                .users
                .into_iter()
                .map(|user| user.into())
                .collect(),
            count: list_response.count,
        }
    }
}
