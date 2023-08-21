use madtofan_microservice_common::user::UserResponse;
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
