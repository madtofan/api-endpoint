use serde::{Deserialize, Serialize};

pub mod notification;
pub mod templating;
pub mod user;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct StatusMessageResponse {
    pub status: String,
}
