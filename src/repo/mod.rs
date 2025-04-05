pub mod user {
    use sqlx::Row;

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    pub struct InsertedData {
        pub id: uuid::Uuid,
        pub date_created: Option<time::OffsetDateTime>,
    }

    pub async fn exists(pool: &sqlx::PgPool, username: &String) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
        SELECT 1 FROM "user" WHERE username = $1
        "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await;

        match result {
            Ok(r) => Ok(r.is_some()),
            Err(e) => Err(e),
        }
    }

    pub async fn insert(
        pool: &sqlx::PgPool,
        user: &icarus_models::user::User,
    ) -> Result<uuid::Uuid, sqlx::Error> {
        let row = sqlx::query(
            r#"
                INSERT INTO "user" (username, password, email, phone, firstname, lastname, email_verified, status, salt_id) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING id, date_created;
            "#)
            .bind(&user.username)
            .bind(&user.password)
            .bind(&user.email)
            .bind(&user.phone)
            .bind(&user.firstname)
            .bind(&user.lastname)
            .bind(user.email_verified)
            .bind(&user.status)
            .bind(user.salt_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            eprintln!("Error inserting item: {}", e);
            e
        })?;

        let result = InsertedData {
            id: row.try_get("id").map_err(|_e| sqlx::Error::RowNotFound)?,
            date_created: row
                .try_get("date_created")
                .map_err(|_e| sqlx::Error::RowNotFound)?,
        };

        if !result.id.is_nil() {
            Ok(result.id)
        } else {
            Err(sqlx::Error::RowNotFound)
        }
    }
}

pub mod salt {
    use sqlx::Row;

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    pub struct InsertedData {
        pub id: uuid::Uuid,
    }

    pub async fn insert(
        pool: &sqlx::PgPool,
        salt: &icarus_models::user::salt::Salt,
    ) -> Result<uuid::Uuid, sqlx::Error> {
        let row = sqlx::query(
            r#"
                INSERT INTO "salt" (salt) 
                VALUES ($1)
                RETURNING id;
            "#,
        )
        .bind(&salt.salt)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            eprintln!("Error inserting item: {}", e);
            e
        })?;

        let result = InsertedData {
            id: row.try_get("id").map_err(|_e| sqlx::Error::RowNotFound)?,
        };

        if !result.id.is_nil() {
            Ok(result.id)
        } else {
            Err(sqlx::Error::RowNotFound)
        }
    }
}
