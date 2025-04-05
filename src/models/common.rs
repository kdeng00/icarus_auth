use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: uuid::Uuid,
    pub username: String,
    pub password: String,
}
