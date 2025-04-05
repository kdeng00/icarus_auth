pub mod user {
    use crate::models;

    pub async fn insert(
        pool: &sqlx::PgPool,
        user: &models::common::User,
    ) -> Result<uuid::Uuid, sqlx::Error> {
        let insert_sql = "INSERT INTO \"user\" (username, password) VALUES ($1, $2) RETURNING id";

        match sqlx::query_scalar(insert_sql)
            .bind(&user.username) // Bind the input message securely
            .bind(&user.password)
            .fetch_one(pool) // Execute and expect exactly ONE row with ONE column back
            .await
        {
            Ok(o) => Ok(o),
            Err(err) => Err(err), // _ => uuid::Uuid::nil(),
        }
    }
}
