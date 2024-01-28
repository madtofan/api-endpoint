use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod notification;
pub mod templating;
pub mod user;

#[derive(Serialize, Deserialize, Default, Debug, TS)]
#[ts(export)]
pub struct StatusMessageResponse {
    pub status: String,
}
