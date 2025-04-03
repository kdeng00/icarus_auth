use axum::{
    Router,
    routing::{get, post},
};
// use std::net::SocketAddr;

use icarus_auth::callers;
use icarus_auth::config;
// use sqlx::Postgres;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let app = app().await;

    // run our app with hyper, listening globally on port 3000
    let url = config::get_full();
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn app() -> Router {
    let pool = icarus_auth::db_pool::create_pool()
        .await
        .expect("Failed to create pool");

    // build our application with a route
    Router::new()
        .route(callers::endpoints::DBTEST, get(callers::common::db_ping))
        .route(callers::endpoints::ROOT, get(callers::common::root))
        .route(
            callers::endpoints::REGISTER,
            post(callers::register::register_user),
        )
        .layer(axum::Extension(pool))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        // extract::connect_info::MockConnectInfo,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt;
    // use http_body_util::BodyExt; // for `collect`
    // use serde_json::{Value, json};
    // use tokio::net::TcpListener;
    // use tower::{Service, ServiceExt}; // for `call`, `oneshot`, and `ready`
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

    #[tokio::test]
    async fn hello_world() {
        let app = app().await;

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

        /*
        match response.into_body().collect().await {
            Ok(o) => {
                let parsed: String = match String::from_utf8(o.to_bytes()) {
                    Ok(s) => s,
                    Err(err) => {
                        String::new()
                    }
                };
            }
            Err(err) => {
                assert!(false,
                "Error: {:?}", err.to_string());
            }
        }
        */

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Hello, World!");
    }
}
