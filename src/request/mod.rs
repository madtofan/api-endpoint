use serde::Deserialize;

pub mod notification;
pub mod templating;
pub mod user;

#[derive(Deserialize)]
pub struct Pagination {
    pub page: i64,
}
