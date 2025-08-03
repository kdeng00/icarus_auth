pub mod request {
    use serde::{Deserialize, Serialize};

    #[derive(Default, Deserialize, Serialize)]
    pub struct Request {
        pub username: String,
        pub password: String,
    }

    pub mod service_login {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct Request {
            pub passphrase: String,
        }
    }
}

pub mod response {
    use serde::{Deserialize, Serialize};

    #[derive(Default, Deserialize, Serialize)]
    pub struct Response {
        pub message: String,
        pub data: Vec<icarus_models::login_result::LoginResult>,
    }

    pub mod service_login {
        #[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
        pub struct Response {
            pub message: String,
            pub data: Vec<icarus_models::login_result::LoginResult>,
        }
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
        // Check if user exists
        match repo::user::get(&pool, &payload.username).await {
            Ok(user) => {
                if hashing::verify_password(&payload.password, user.password.clone()).unwrap() {
                    // Create token
                    let key = icarus_envy::environment::get_secret_key().await;
                    let (token_literal, duration) = token_stuff::create_token(&key).unwrap();

                    if token_stuff::verify_token(&key, &token_literal) {
                        let current_time = time::OffsetDateTime::now_utc();
                        let _ = repo::user::update_last_login(&pool, &user, &current_time).await;

                        (
                            StatusCode::OK,
                            Json(response::Response {
                                message: String::from("Successful"),
                                data: vec![icarus_models::login_result::LoginResult {
                                    id: user.id,
                                    username: user.username.clone(),
                                    token: token_literal,
                                    token_type: String::from(icarus_models::token::TOKEN_TYPE),
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

    pub async fn service_login(
        axum::Extension(pool): axum::Extension<sqlx::PgPool>,
        axum::Json(payload): axum::Json<request::service_login::Request>,
    ) -> (
        axum::http::StatusCode,
        axum::Json<response::service_login::Response>,
    ) {
        let mut response = response::service_login::Response::default();

        match repo::service::valid_passphrase(&pool, &payload.passphrase).await {
            Ok((id, _passphrase, _date_created)) => {
                let key = icarus_envy::environment::get_secret_key().await;
                let (token_literal, duration) = token_stuff::create_service_token(&key).unwrap();

                if token_stuff::verify_token(&key, &token_literal) {
                    let login_result = icarus_models::login_result::LoginResult {
                        id,
                        username: String::from("service"),
                        token: token_literal,
                        token_type: String::from(icarus_models::token::TOKEN_TYPE),
                        expiration: duration,
                    };

                    response.data.push(login_result);
                    response.message = String::from("Successful");

                    (axum::http::StatusCode::OK, axum::Json(response))
                } else {
                    (axum::http::StatusCode::OK, axum::Json(response))
                }
            }
            Err(err) => {
                response.message = err.to_string();
                (axum::http::StatusCode::BAD_REQUEST, axum::Json(response))
            }
        }
    }
}
