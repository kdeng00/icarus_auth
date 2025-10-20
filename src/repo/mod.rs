pub mod user {
    use sqlx::Row;

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    pub struct InsertedData {
        pub id: uuid::Uuid,
        pub date_created: Option<time::OffsetDateTime>,
    }

    pub async fn get(
        pool: &sqlx::PgPool,
        username: &String,
    ) -> Result<icarus_models::user::User, sqlx::Error> {
        let result = sqlx::query(
            r#"
        SELECT * FROM "user" WHERE username = $1
        "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await;

        match result {
            Ok(r) => match r {
                Some(r) => Ok(icarus_models::user::User {
                    id: r.try_get("id")?,
                    username: r.try_get("username")?,
                    password: r.try_get("password")?,
                    email: r.try_get("email")?,
                    email_verified: r.try_get("email_verified")?,
                    phone: r.try_get("phone")?,
                    salt_id: r.try_get("salt_id")?,
                    firstname: r.try_get("firstname")?,
                    lastname: r.try_get("lastname")?,
                    date_created: r.try_get("date_created")?,
                    last_login: r.try_get("last_login")?,
                    status: r.try_get("status")?,
                }),
                None => Err(sqlx::Error::RowNotFound),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn update_last_login(
        pool: &sqlx::PgPool,
        user: &icarus_models::user::User,
        time: &time::OffsetDateTime,
    ) -> Result<time::OffsetDateTime, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE "user" SET last_login = $1 WHERE id = $2 RETURNING last_login
            "#,
        )
        .bind(time)
        .bind(user.id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            eprintln!("Error updating time: {e}");
            e
        });

        match result {
            Ok(row) => match row {
                Some(r) => {
                    let last_login: time::OffsetDateTime = r
                        .try_get("last_login")
                        .map_err(|_e| sqlx::Error::RowNotFound)?;
                    Ok(last_login)
                }
                None => Err(sqlx::Error::RowNotFound),
            },
            Err(err) => Err(err),
        }
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
    ) -> Result<(uuid::Uuid, std::option::Option<time::OffsetDateTime>), sqlx::Error> {
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
            eprintln!("Error inserting item: {e}");
            e
        })?;

        let result = InsertedData {
            id: row.try_get("id").map_err(|_e| sqlx::Error::RowNotFound)?,
            date_created: row
                .try_get("date_created")
                .map_err(|_e| sqlx::Error::RowNotFound)?,
        };

        if result.id.is_nil() && result.date_created.is_none() {
            Err(sqlx::Error::RowNotFound)
        } else {
            Ok((result.id, result.date_created))
        }
    }
}

pub mod salt {
    use sqlx::Row;

    #[derive(Debug, serde::Serialize, sqlx::FromRow)]
    pub struct InsertedData {
        pub id: uuid::Uuid,
    }

    pub async fn get(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
    ) -> Result<icarus_models::user::salt::Salt, sqlx::Error> {
        let result = sqlx::query(
            r#"
        SELECT * FROM "salt" WHERE id = $1
        "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await;

        match result {
            Ok(r) => match r {
                Some(r) => Ok(icarus_models::user::salt::Salt {
                    id: r.try_get("id")?,
                    salt: r.try_get("salt")?,
                }),
                None => Err(sqlx::Error::RowNotFound),
            },
            Err(e) => Err(e),
        }
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
            eprintln!("Error inserting item: {e}");
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

pub mod service {
    use sqlx::Row;

    pub async fn valid_passphrase(
        pool: &sqlx::PgPool,
        passphrase: &String,
    ) -> Result<(uuid::Uuid, String, time::OffsetDateTime), sqlx::Error> {
        let result = sqlx::query(
            r#"
            SELECT * FROM "passphrase" WHERE passphrase = $1
            "#,
        )
        .bind(passphrase)
        .fetch_one(pool)
        .await;

        match result {
            Ok(row) => {
                let id: uuid::Uuid = row.try_get("id")?;
                let passphrase: String = row.try_get("passphrase")?;
                let date_created: Option<time::OffsetDateTime> = row.try_get("date_created")?;

                Ok((id, passphrase, date_created.unwrap()))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_passphrase(
        pool: &sqlx::PgPool,
        id: &uuid::Uuid,
    ) -> Result<(uuid::Uuid, String, time::OffsetDateTime), sqlx::Error> {
        let result = sqlx::query(
            r#"
            SELECT * FROM "passphrase" WHERE id = $1;
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await;

        match result {
            Ok(row) => {
                let returned_id: uuid::Uuid = row.try_get("id")?;
                let passphrase: String = row.try_get("passphrase")?;
                let date_created: time::OffsetDateTime = row.try_get("date_created")?;
                Ok((returned_id, passphrase, date_created))
            }
            Err(err) => Err(err),
        }
    }
}
