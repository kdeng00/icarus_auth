use axum::{
    Router,
    routing::{get, post},
};

use icarus_auth::callers;
use icarus_auth::config;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route(callers::endpoints::ROOT, get(callers::common::root))
        .route(
            callers::endpoints::REGISTER,
            post(callers::register::register_user),
        );

    // run our app with hyper, listening globally on port 3000
    let url = config::get_full();
    let listener = tokio::net::TcpListener::bind(url).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
