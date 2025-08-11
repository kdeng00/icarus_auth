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

    pub mod refresh_token {
        #[derive(Debug, serde::Deserialize, serde::Serialize)]
        pub struct Request {
            pub access_token: String,
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

    pub mod refresh_token {
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

    // TODO: At some point, get the username from the DB
    // Name of service username when returning a login result
    pub const SERVICE_USERNAME: &str = "service";

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
                    let (token_literal, duration) =
                        token_stuff::create_token(&key, &user.id).unwrap();

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
                let (token_literal, duration) =
                    token_stuff::create_service_token(&key, &id).unwrap();

                if token_stuff::verify_token(&key, &token_literal) {
                    let login_result = icarus_models::login_result::LoginResult {
                        id,
                        username: String::from(SERVICE_USERNAME),
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

    pub async fn refresh_token(
        axum::Extension(pool): axum::Extension<sqlx::PgPool>,
        axum::Json(payload): axum::Json<request::refresh_token::Request>,
    ) -> (
        axum::http::StatusCode,
        axum::Json<response::refresh_token::Response>,
    ) {
        let mut response = response::refresh_token::Response::default();
        let key = icarus_envy::environment::get_secret_key().await;

        if token_stuff::verify_token(&key, &payload.access_token) {
            let token_type = token_stuff::get_token_type(&key, &payload.access_token).unwrap();

            if token_stuff::is_token_type_valid(&token_type) {
                // Get passphrase record with id
                match token_stuff::extract_id_from_token(&key, &payload.access_token) {
                    Ok(id) => match repo::service::get_passphrase(&pool, &id).await {
                        Ok((returned_id, _, _)) => {
                            match token_stuff::create_service_refresh_token(&key, &returned_id) {
                                Ok((access_token, exp_dur)) => {
                                    let login_result = icarus_models::login_result::LoginResult {
                                        id: returned_id,
                                        token: access_token,
                                        expiration: exp_dur,
                                        token_type: String::from(icarus_models::token::TOKEN_TYPE),
                                        username: String::from(SERVICE_USERNAME),
                                    };
                                    response.message = String::from("Successful");
                                    response.data.push(login_result);

                                    (axum::http::StatusCode::OK, axum::Json(response))
                                }
                                Err(err) => {
                                    response.message = err.to_string();
                                    (
                                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                        axum::Json(response),
                                    )
                                }
                            }
                        }
                        Err(err) => {
                            response.message = err.to_string();
                            (
                                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                                axum::Json(response),
                            )
                        }
                    },
                    Err(err) => {
                        response.message = err.to_string();
                        (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            axum::Json(response),
                        )
                    }
                }
            } else {
                response.message = String::from("Invalid token type");
                (axum::http::StatusCode::NOT_FOUND, axum::Json(response))
            }
        } else {
            response.message = String::from("Could not verify token");
            (axum::http::StatusCode::BAD_REQUEST, axum::Json(response))
        }
    }
}
