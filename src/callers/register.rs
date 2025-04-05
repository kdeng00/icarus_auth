use axum::{Json, http::StatusCode};

use crate::models;
use crate::repo;

mod response {
    use serde::{Deserialize, Serialize};

    use crate::models;

    #[derive(Deserialize, Serialize)]
    pub struct Response {
        pub message: String,
        pub data: models::common::User,
    }
}

pub async fn register_user(
    axum::Extension(pool): axum::Extension<sqlx::PgPool>,
    Json(payload): Json<models::common::CreateUser>,
) -> (StatusCode, Json<response::Response>) {
    let mut user = models::common::User {
        id: uuid::Uuid::nil(),
        username: payload.username.clone(),
        password: payload.password.clone(),
    };

    match repo::user::insert(&pool, &user).await {
        Ok(id) => {
            user.id = id;
            (
                StatusCode::CREATED,
                Json(response::Response {
                    message: String::from("User inserted"),
                    data: user,
                }),
            )
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(response::Response {
                message: err.to_string(),
                data: user,
            }),
        ),
    }
}
