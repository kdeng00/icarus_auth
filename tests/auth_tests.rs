extern crate icarus_auth;

use axum::body::Body;
// use axum::response::Response;
use axum::{
    Router,
    http::{Request, StatusCode},
    routing::get,
};
// use http::{Request, StatusCode};
// use serde_json::json;
// use tower::ServiceExt; // for `.oneshot()`
use tower::util::ServiceExt;

use crate::icarus_auth::callers;

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
