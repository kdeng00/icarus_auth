pub mod response {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize)]
    pub struct TestResult {
        pub message: String,
    }
}

pub mod endpoint {
    use super::*;
    use axum::{Extension, Json, http::StatusCode};

    // basic handler that responds with a static string
    pub async fn root() -> &'static str {
        "Hello, World!"
    }

    pub async fn db_ping(
        Extension(pool): Extension<sqlx::PgPool>,
    ) -> (StatusCode, Json<response::TestResult>) {
        match sqlx::query("SELECT 1").execute(&pool).await {
            Ok(_) => {
                let tr = response::TestResult {
                    message: String::from("This works"),
                };
                (StatusCode::OK, Json(tr))
            }
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(response::TestResult {
                    message: e.to_string(),
                }),
            ),
        }
    }
}
