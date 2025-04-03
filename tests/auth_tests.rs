extern crate icarus_auth;

use crate::icarus_auth::callers;

// use axum::Extension;
use axum::body::Body;
// use axum::response::Response;
use axum::{
    Router,
    http::{Request, StatusCode},
    routing::get,
};
// use hyper::client::conn;
// use sqlx::PgPool;
// use sqlx::postgres::{self, PgPoolOptions};
// use testcontainers_modules::testcontainers::runners::AsyncRunner;
// use hyper::client;
// use sqlx::postgres;
// use http::{Request, StatusCode};
// use serde_json::json;
// use tower::ServiceExt; // for `.oneshot()`
use tower::util::ServiceExt;
// use testcontainers_modules::testcontainers::core::client::

const TEST_DATABASE_URL_ENV: &str = "TEST_DATABASE_URL";
const DEFAULT_TEST_DATABASE_URL: &str =
    "postgres://icarus_op_test:password@localhost:5432/icarus_auth_test";

static SETUP: std::sync::Once = std::sync::Once::new();

// Ensure tracing is initialized only once for all tests in this file
/*
static TRACING_INIT: Lazy<()> = Lazy::new(|| {
    if std::env::var("RUST_LOG").is_err() {
        // Set default log level if not provided
        unsafe {
        std::env::set_var("RUST_LOG", "info,tower_http=debug,your_project_name=debug");
        }
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_test_writer() // Write logs to the test output capture
        .init();
});
*/

/*
async fn setup_database() -> sqlx::PgPool {
    let database_url = std::env::var(TEST_DATABASE_URL_ENV)
        .unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    let migrator = sqlx::migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("Failed to create migrator");
    migrator.run(&pool).await.expect("Failed to run migrations");

    // Seed here if needed
    pool
}
    */

/*
#[tokio::test]
async fn test_db_health() {
    SETUP.call_once(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            setup_database().await;
        });
    });
}
*/

/*
async fn setup_test(pool: sqlx::PgPool) -> Router {
    Router::new()
        .route(callers::endpoints::DBTEST, get(callers::common::db_ping))
        .layer(Extension(pool))
}
*/

/*
#[tokio::test]
async fn test_hello_world() {
    let app = Router::new().route(callers::endpoints::ROOT, get(callers::common::root)); // Replace with your handler

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

    let body = String::from_utf8(
        axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();

    assert_eq!(body, "Hello, World!");
}
*/

/*
#[tokio::test]
async fn _test_db_health_check() {
    let container = testcontainers_modules::postgres::Postgres::default()
        .start()
        .await
        .unwrap();
    let _host_ip = container.get_host().await.unwrap();
    let port = 5432;
    let host_port = container.get_host_port_ipv4(port).await.unwrap();
    let conn_string = &format!(
        "postgres://postgres:postgres@localhost:{}/postgres",
        host_port
    );

    println!("Test Database: {}", conn_string);

    let app = Router::new().route(callers::endpoints::DBTEST, get(callers::common::db_ping)); // Replace with your handler

    let response = app
        .oneshot(
            Request::builder()
                .uri(callers::endpoints::DBTEST)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    match PgPoolOptions::new().connect(conn_string).await {
        Ok(_) => {
            assert!(true, "Success");
        }
        Err(err) => {
            assert!(false, "Error: {:?}", err.to_string());
        }
    };
}
    */
