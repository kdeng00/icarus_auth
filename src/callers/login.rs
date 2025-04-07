pub mod request {
    use serde::{Deserialize, Serialize};

    #[derive(Default, Deserialize, Serialize)]
    pub struct Request {
        pub username: String,
        pub password: String,
    }
}

pub mod response {
    use serde::{Deserialize, Serialize};

    #[derive(Default, Deserialize, Serialize)]
    pub struct Response {
        pub message: String,
        pub data: Vec<icarus_models::login_result::LoginResult>,
    }
}

pub mod endpoint {
    use axum::{Json, http::StatusCode};

    use crate::hashing;
    use crate::repo;
    use crate::token_stuff;

    use super::request;
    use super::response;

    async fn not_found(message: &str) -> (StatusCode, Json<response::Response>) {
        (
            StatusCode::NOT_FOUND,
            Json(response::Response {
                message: String::from(message),
                data: Vec::new(),
            }),
        )
    }

    pub async fn login(
        axum::Extension(pool): axum::Extension<sqlx::PgPool>,
        Json(payload): Json<request::Request>,
    ) -> (StatusCode, Json<response::Response>) {
        let usr = icarus_models::user::User {
            username: payload.username,
            password: payload.password,
            ..Default::default()
        };

        // Check if user exists
        match repo::user::exists(&pool, &usr.username).await {
            Ok(exists) => {
                if !exists {
                    return not_found("Not Found").await;
                }
            }
            Err(err) => {
                return not_found(&err.to_string()).await;
            }
        };

        let user = repo::user::get(&pool, &usr.username).await.unwrap();
        let salt = repo::salt::get(&pool, &user.salt_id).await.unwrap();
        let salt_str = hashing::get_salt(&salt.salt).unwrap();

        // Check if password is correct
        match hashing::hash_password(&usr.password, &salt_str) {
            Ok(hash_password) => {
                if hashing::verify_password(&usr.password, hash_password.clone()).unwrap() {
                    // Create token
                    let key = token_stuff::get_key().unwrap();
                    let (token_literal, duration) = token_stuff::create_token(&key).unwrap();

                    if token_stuff::verify_token(&key, &token_literal) {
                        (
                            StatusCode::OK,
                            Json(response::Response {
                                message: String::from("Successful"),
                                data: vec![icarus_models::login_result::LoginResult {
                                    id: user.id,
                                    username: user.username,
                                    token: token_literal,
                                    token_type: String::from(token_stuff::TOKENTYPE),
                                    expiration: duration,
                                }],
                            }),
                        )
                    } else {
                        return not_found("Could not verify password").await;
                    }
                } else {
                    return not_found("Error Hashing").await;
                }
            }
            Err(err) => {
                return not_found(&err.to_string()).await;
            }
        }
    }
}
