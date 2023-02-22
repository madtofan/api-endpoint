use serde::{Deserialize, Serialize};

use crate::user::UserResponse;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserEndpointResponse {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i64,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub token: String,
}

impl UserEndpointResponse {
    pub fn from_user_response(user_response: UserResponse, token: String) -> Self {
        Self {
            id: user_response.id,
            username: user_response.username,
            email: user_response.email,
            bio: user_response.bio,
            image: user_response.image,
            token,
        }
    }
}
