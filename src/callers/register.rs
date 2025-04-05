use axum::{Json, http::StatusCode};

use crate::hashing;
use crate::repo;

pub mod request {
    use serde::{Deserialize, Serialize};

    #[derive(Default, Deserialize, Serialize)]
    pub struct Request {
        #[serde(skip_serializing_if = "String::is_empty")]
        pub username: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        pub password: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        pub email: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        pub phone: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        pub firstname: String,
        #[serde(skip_serializing_if = "String::is_empty")]
        pub lastname: String,
    }
}

pub mod response {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct Response {
        pub message: String,
        pub data: Vec<icarus_models::user::User>,
    }
}

pub async fn register_user(
    axum::Extension(pool): axum::Extension<sqlx::PgPool>,
    Json(payload): Json<request::Request>,
) -> (StatusCode, Json<response::Response>) {
    let mut user = icarus_models::user::User {
        id: uuid::Uuid::nil(),
        username: payload.username.clone(),
        password: payload.password.clone(),
        email: payload.email.clone(),
        phone: payload.phone.clone(),
        firstname: payload.firstname.clone(),
        lastname: payload.lastname.clone(),
        status: String::from("Active"),
        email_verified: true,
        date_created: Some(time::OffsetDateTime::now_utc()),
        last_login: None,
        salt_id: uuid::Uuid::nil(),
    };

    match repo::user::exists(&pool, &user.username).await {
        Ok(res) => {
            if res {
                (
                    StatusCode::NOT_FOUND,
                    Json(response::Response {
                        message: String::from("Error"),
                        data: vec![user],
                    }),
                )
            } else {
                let salt_string = hashing::generate_salt().unwrap();
                let mut salt = icarus_models::user::salt::Salt::default();
                let generated_salt = salt_string;
                salt.salt = generated_salt.to_string();
                salt.id = repo::salt::insert(&pool, &salt).await.unwrap();
                user.salt_id = salt.id;
                let hashed_password =
                    hashing::hash_password(&user.password, &generated_salt).unwrap();
                user.password = hashed_password;

                match repo::user::insert(&pool, &user).await {
                    Ok(id) => {
                        user.id = id;
                        (
                            StatusCode::CREATED,
                            Json(response::Response {
                                message: String::from("User created"),
                                data: vec![user],
                            }),
                        )
                    }
                    Err(err) => (
                        StatusCode::BAD_REQUEST,
                        Json(response::Response {
                            message: err.to_string(),
                            data: vec![user],
                        }),
                    ),
                }
            }
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(response::Response {
                message: err.to_string(),
                data: vec![user],
            }),
        ),
    }
}
