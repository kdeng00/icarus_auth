pub mod callers;
pub mod config;
pub mod hashing;
pub mod repo;
pub mod token_stuff;

pub mod keys {
    pub const DBURL: &str = "DATABASE_URL";

    pub mod error {
        pub const ERROR: &str = "DATABASE_URL must be set in .env";
    }
}

mod connection_settings {
    pub const MAXCONN: u32 = 5;
}

pub mod db {

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

    pub async fn migrations(pool: &sqlx::PgPool) {
        // Run migrations using the sqlx::migrate! macro
        // Assumes your migrations are in a ./migrations folder relative to Cargo.toml
        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .expect("Failed to run migrations");
    }
}
