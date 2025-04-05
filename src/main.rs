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

mod db {
    pub async fn migrations(pool: &sqlx::PgPool) {
        // Run migrations using the sqlx::migrate! macro
        // Assumes your migrations are in a ./migrations folder relative to Cargo.toml
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .expect("Failed to run migrations on testcontainer DB");
    }
}

mod init {
    use axum::{
        Router,
        routing::{get, post},
    };

    use crate::callers;
    use crate::db;

    pub async fn routes() -> Router {
        // build our application with a route
        Router::new()
            .route(callers::endpoints::DBTEST, get(callers::common::db_ping))
            .route(callers::endpoints::ROOT, get(callers::common::root))
            .route(
                callers::endpoints::REGISTER,
                post(callers::register::register_user),
            )
    }

    pub async fn app() -> Router {
        let pool = icarus_auth::db_pool::create_pool()
            .await
            .expect("Failed to create pool");

        db::migrations(&pool).await;

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
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

    #[derive(Deserialize, Serialize)]
    struct Response {
        pub message: String,
        pub data: icarus_auth::models::common::User,
    }

    mod db_mgr {
        use std::str::FromStr;

        use icarus_auth::keys;

        pub const LIMIT: usize = 6;

        pub async fn get_pool() -> Result<sqlx::PgPool, sqlx::Error> {
            let tm_db_url = std::env::var(keys::DBURL).expect("DATABASE_URL must be present");
            let tm_options = sqlx::postgres::PgConnectOptions::from_str(&tm_db_url).unwrap();
            sqlx::PgPool::connect_with(tm_options).await
        }

        pub async fn generate_db_name() -> String {
            let db_name =
                get_database_name().unwrap() + &"_" + &uuid::Uuid::new_v4().to_string()[..LIMIT];
            db_name
        }

        pub async fn connect_to_db(db_name: &str) -> Result<sqlx::PgPool, sqlx::Error> {
            let db_url = std::env::var(keys::DBURL).expect("DATABASE_URL must be set for tests");
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

        pub fn get_database_name() -> Result<String, Box<dyn std::error::Error>> {
            dotenvy::dotenv().ok(); // Load .env file if it exists

            match std::env::var(keys::DBURL) {
                Ok(database_url) => {
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
                Err(_) => {
                    // DATABASE_URL environment variable not found
                    Err("Error parsing".into())
                }
            }
        }
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

        db::migrations(&pool).await;

        let app = init::routes().await.layer(axum::Extension(pool));

        let usr = icarus_auth::models::common::CreateUser {
            username: String::from("somethingsss"),
            password: String::from("Raindown!"),
        };

        let payload = json!({
            "username": &usr.username,
            "password": &usr.password,
        });

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
                assert_eq!(resp.status(), StatusCode::CREATED, "Message: {:?}", resp);
                let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let parsed_body: Response = serde_json::from_slice(&body).unwrap();

                assert_eq!(
                    usr.username, parsed_body.data.username,
                    "Usernames do not match"
                );
            }
            Err(err) => {
                assert!(false, "Error: {:?}", err.to_string());
            }
        };

        let _ = db_mgr::drop_database(&tm_pool, &db_name).await;
    }
}
