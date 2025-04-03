use axum::{Extension, Json, http::StatusCode};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TestResult {
    message: String,
}

// basic handler that responds with a static string
pub async fn root() -> &'static str {
    "Hello, World!"
}

pub async fn db_ping(Extension(pool): Extension<sqlx::PgPool>) -> (StatusCode, Json<TestResult>) {
    match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => {
            let tr = TestResult {
                message: String::from("This works"),
            };
            (StatusCode::OK, Json(tr))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(TestResult {
                message: e.to_string(),
            }),
        ),
    }
}
