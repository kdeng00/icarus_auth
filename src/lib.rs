pub mod callers;
pub mod config;
pub mod hashing;
pub mod models;
pub mod repo;

pub mod keys {
    pub const DBURL: &str = "DATABASE_URL";

    pub mod error {
        pub const ERROR: &str = "DATABASE_URL must be set in .env";
    }
}

mod connection_settings {
    pub const MAXCONN: u32 = 5;
}

pub mod db_pool {

    use sqlx::postgres::PgPoolOptions;
    use std::env;

    use crate::{connection_settings, keys};

    pub async fn create_pool() -> Result<sqlx::PgPool, sqlx::Error> {
        let database_url = get_db_url().await;
        println!("Database url: {:?}", database_url);

        PgPoolOptions::new()
            .max_connections(connection_settings::MAXCONN)
            .connect(&database_url)
            .await
    }

    async fn get_db_url() -> String {
        #[cfg(debug_assertions)] // Example: Only load .env in debug builds
        dotenvy::dotenv().ok();

        env::var(keys::DBURL).expect(keys::error::ERROR)
    }
}
