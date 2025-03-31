use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

#[derive(Serialize)]
pub struct User {
    pub username: String,
}
