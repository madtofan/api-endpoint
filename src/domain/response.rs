use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserDto {
    #[serde(skip_serializing, skip_deserializing)]
    pub id: i64,
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub token: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UserAuthenticationResponse {
    pub user: UserDto,
}

impl UserAuthenticationResponse {
    pub fn new(
        id: i64,
        username: String,
        email: String,
        bio: Option<String>,
        image: Option<String>,
        token: String,
    ) -> Self {
        UserAuthenticationResponse {
            user: UserDto {
                id,
                username,
                email,
                bio,
                image,
                token,
            },
        }
    }
}
