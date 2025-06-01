use icarus_auth::callers;
use icarus_auth::config;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let app = init::app().await;

    // run our app with hyper, listening globally on port 3000
    let url = config::get_full();
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

mod init {
    use axum::{
        Router,
        routing::{get, post},
    };

    use crate::callers;

    pub async fn routes() -> Router {
        // build our application with a route
        Router::new()
            .route(
                callers::endpoints::DBTEST,
                get(callers::common::endpoint::db_ping),
            )
            .route(
                callers::endpoints::ROOT,
                get(callers::common::endpoint::root),
            )
            .route(
                callers::endpoints::REGISTER,
                post(callers::register::register_user),
            )
            .route(
                callers::endpoints::LOGIN,
                post(callers::login::endpoint::login),
            )
    }

    pub async fn app() -> Router {
        let pool = icarus_auth::db::create_pool()
            .await
            .expect("Failed to create pool");

        icarus_auth::db::migrations(&pool).await;

        routes().await.layer(axum::Extension(pool))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use serde_json::json;
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

    mod db_mgr {
        use std::str::FromStr;

        pub const LIMIT: usize = 6;

        pub async fn get_pool() -> Result<sqlx::PgPool, sqlx::Error> {
            let tm_db_url = icarus_envy::environment::get_db_url().await;
            let tm_options = sqlx::postgres::PgConnectOptions::from_str(&tm_db_url).unwrap();
            sqlx::PgPool::connect_with(tm_options).await
        }

        pub async fn generate_db_name() -> String {
            let db_name = get_database_name().await.unwrap()
                + &"_"
                + &uuid::Uuid::new_v4().to_string()[..LIMIT];
            db_name
        }

        pub async fn connect_to_db(db_name: &str) -> Result<sqlx::PgPool, sqlx::Error> {
            let db_url = icarus_envy::environment::get_db_url().await;
            let options = sqlx::postgres::PgConnectOptions::from_str(&db_url)?.database(db_name);
            sqlx::PgPool::connect_with(options).await
        }

        pub async fn create_database(
            template_pool: &sqlx::PgPool,
            db_name: &str,
        ) -> Result<(), sqlx::Error> {
            let create_query = format!("CREATE DATABASE {}", db_name);
            match sqlx::query(&create_query).execute(template_pool).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        }

        // Function to drop a database
        pub async fn drop_database(
            template_pool: &sqlx::PgPool,
            db_name: &str,
        ) -> Result<(), sqlx::Error> {
            let drop_query = format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", db_name);
            sqlx::query(&drop_query).execute(template_pool).await?;
            Ok(())
        }

        pub async fn get_database_name() -> Result<String, Box<dyn std::error::Error>> {
            let database_url = icarus_envy::environment::get_db_url().await;

            let parsed_url = url::Url::parse(&database_url)?;
            if parsed_url.scheme() == "postgres" || parsed_url.scheme() == "postgresql" {
                match parsed_url
                    .path_segments()
                    .and_then(|segments| segments.last().map(|s| s.to_string()))
                {
                    Some(sss) => Ok(sss),
                    None => Err("Error parsing".into()),
                }
            } else {
                // Handle other database types if needed
                Err("Error parsing".into())
            }
        }
    }

    fn get_test_register_request() -> icarus_auth::callers::register::request::Request {
        icarus_auth::callers::register::request::Request {
            username: String::from("somethingsss"),
            password: String::from("Raindown!"),
            email: String::from("dev@null.com"),
            phone: String::from("1234567890"),
            firstname: String::from("Bob"),
            lastname: String::from("Smith"),
        }
    }

    fn get_test_register_payload(
        usr: &icarus_auth::callers::register::request::Request,
    ) -> serde_json::Value {
        json!({
            "username": &usr.username,
            "password": &usr.password,
            "email": &usr.email,
            "phone": &usr.phone,
            "firstname": &usr.firstname,
            "lastname": &usr.lastname,
        })
    }

    #[tokio::test]
    async fn test_hello_world() {
        let app = init::app().await;

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(
                Request::builder()
                    .uri(callers::endpoints::ROOT)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Hello, World!");
    }

    #[tokio::test]
    async fn test_register_user() {
        let tm_pool = db_mgr::get_pool().await.unwrap();

        let db_name = db_mgr::generate_db_name().await;

        match db_mgr::create_database(&tm_pool, &db_name).await {
            Ok(_) => {
                println!("Success");
            }
            Err(e) => {
                assert!(false, "Error: {:?}", e.to_string());
            }
        }

        let pool = db_mgr::connect_to_db(&db_name).await.unwrap();

        icarus_auth::db::migrations(&pool).await;

        let app = init::routes().await.layer(axum::Extension(pool));

        let usr = get_test_register_request();
        let payload = get_test_register_payload(&usr);

        let response = app
            .oneshot(
                Request::builder()
                    .method(axum::http::Method::POST)
                    .uri(callers::endpoints::REGISTER)
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(
                    resp.status(),
                    StatusCode::CREATED,
                    "Message: {:?} {:?}",
                    resp,
                    usr.username
                );
                let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let parsed_body: callers::register::response::Response =
                    serde_json::from_slice(&body).unwrap();
                let returned_usr = &parsed_body.data[0];

                assert_eq!(false, returned_usr.id.is_nil(), "Id is not populated");

                assert_eq!(
                    usr.username, returned_usr.username,
                    "Usernames do not match"
                );
                assert!(returned_usr.date_created.is_some(), "Date Created is empty");
            }
            Err(err) => {
                assert!(false, "Error: {:?}", err.to_string());
            }
        };

        let _ = db_mgr::drop_database(&tm_pool, &db_name).await;
    }

    #[tokio::test]
    async fn test_login_user() {
        let tm_pool = db_mgr::get_pool().await.unwrap();

        let db_name = db_mgr::generate_db_name().await;

        match db_mgr::create_database(&tm_pool, &db_name).await {
            Ok(_) => {
                println!("Success");
            }
            Err(e) => {
                assert!(false, "Error: {:?}", e.to_string());
            }
        }

        let pool = db_mgr::connect_to_db(&db_name).await.unwrap();

        icarus_auth::db::migrations(&pool).await;

        let app = init::routes().await.layer(axum::Extension(pool));

        let usr = get_test_register_request();
        let payload = get_test_register_payload(&usr);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(axum::http::Method::POST)
                    .uri(callers::endpoints::REGISTER)
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(payload.to_string()))
                    .unwrap(),
            )
            .await;

        match response {
            Ok(resp) => {
                assert_eq!(
                    resp.status(),
                    StatusCode::CREATED,
                    "Message: {:?} {:?}",
                    resp,
                    usr.username
                );
                let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let parsed_body: callers::register::response::Response =
                    serde_json::from_slice(&body).unwrap();
                let returned_usr = &parsed_body.data[0];

                assert_eq!(false, returned_usr.id.is_nil(), "Id is not populated");

                assert_eq!(
                    usr.username, returned_usr.username,
                    "Usernames do not match"
                );
                assert!(returned_usr.date_created.is_some(), "Date Created is empty");

                let login_payload = json!({
                    "username": &usr.username,
                    "password": &usr.password,
                });

                match app
                    .oneshot(
                        Request::builder()
                            .method(axum::http::Method::POST)
                            .uri(callers::endpoints::LOGIN)
                            .header(axum::http::header::CONTENT_TYPE, "application/json")
                            .body(Body::from(login_payload.to_string()))
                            .unwrap(),
                    )
                    .await
                {
                    Ok(resp) => {
                        assert_eq!(StatusCode::OK, resp.status(), "Status is not right");
                        let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
                            .await
                            .unwrap();
                        let parsed_body: callers::login::response::Response =
                            serde_json::from_slice(&body).unwrap();
                        let login_result = &parsed_body.data[0];
                        assert!(!login_result.id.is_nil(), "Id is nil");
                    }
                    Err(err) => {
                        assert!(false, "Error: {:?}", err.to_string());
                    }
                }
            }
            Err(err) => {
                assert!(false, "Error: {:?}", err.to_string());
            }
        };

        let _ = db_mgr::drop_database(&tm_pool, &db_name).await;
    }
}
