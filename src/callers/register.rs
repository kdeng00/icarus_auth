use axum::{Json, http::StatusCode};

use crate::models;

pub async fn register_user(
    Json(payload): Json<models::common::CreateUser>,
) -> (StatusCode, Json<models::common::User>) {
    let user = models::common::User {
        username: payload.username.clone(),
    };
    (StatusCode::CREATED, Json(user))
}
