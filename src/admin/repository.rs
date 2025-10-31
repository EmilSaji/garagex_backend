use chrono::Utc;
use eyre::Result;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction}; // <-- Executor is needed
use uuid::Uuid;

use crate::admin::models::AdminUser;
use crate::admin::models::{Garage, GarageUser, NewGarage};

pub struct AdminRepo;

impl AdminRepo {
    pub async fn find_by_username(pool: &PgPool, username: &str) -> Result<AdminUser> {
        let rec = sqlx::query_as::<_, AdminUser>(
            r#"
            SELECT id, username, password_hash, phone, display_name, email, is_active
            FROM system_users
            WHERE username = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(username)
        .fetch_one(pool)
        .await?;
        Ok(rec)
    }

    // pub async fn create_admin(pool: &PgPool, username: &str, raw_password: &str) -> Result<Uuid> {
    //     let id = Uuid::new_v4();
    //     sqlx::query!(
    //         r#"
    //         INSERT INTO system_users (id, username, password_hash, is_active, created_at)
    //         VALUES ($1, $2, $3, true, now())
    //         "#,
    //         id,
    //         username,
    //         raw_password
    //     )
    //     .execute(pool)
    //     .await?;
    //     Ok(id)
    // }
}

pub struct GarageRepo;

impl GarageRepo {
    /// List garages optionally filtered by a query string that matches name/address.
    pub async fn list_garages(pool: &PgPool, q: Option<&str>, limit: i64) -> Result<Vec<Garage>> {
        if let Some(q) = q {
            let like = format!("%{}%", q);
            let rows = sqlx::query_as::<_, Garage>(
                r#"
                SELECT id, name, address, phone, email, metadata, created_at, updated_at
                FROM garages
                WHERE (name ILIKE $1 OR address ILIKE $1 OR COALESCE((metadata->>'owner'), '') ILIKE $1)
                  AND deleted_at IS NULL
                ORDER BY created_at DESC
                LIMIT $2
                "#,
            )
                .bind(like)
                .bind(limit)
                .fetch_all(pool)
                .await?;
            Ok(rows)
        } else {
            let rows = sqlx::query_as::<_, Garage>(
                r#"
                SELECT id, name, address, phone, email, metadata, created_at, updated_at
                FROM garages
                WHERE deleted_at IS NULL
                ORDER BY created_at DESC
                LIMIT $1
                "#,
            )
            .bind(limit)
            .fetch_all(pool)
            .await?;
            Ok(rows)
        }
    }

    pub async fn add_garage_with_admin(
        pool: &PgPool,
        new: &NewGarage,
    ) -> Result<(Garage, GarageUser)> {
        // start a transaction
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(|e| eyre::eyre!(e))?;

        // insert garage
        let gid = Uuid::new_v4();
        let now = Utc::now();

        let garage: Garage = sqlx::query_as::<_, Garage>(
            r#"
    INSERT INTO garages
        (id, name, address, phone, email, metadata, created_at, updated_at, deleted_at)
    VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, NULL)
    RETURNING id, name, address, phone, email, metadata, created_at, updated_at
    "#,
        )
            .bind(gid)
            .bind(&new.name)
            .bind(&new.address)
            .bind(&new.phone)
            .bind(&new.email)
            .bind(&new.metadata)
            .bind(now)
            .bind(None::<chrono::DateTime<Utc>>)
            .fetch_one(&mut *tx) // Single dereference
            .await
            .map_err(|e| eyre::eyre!(e))?;

        // insert placeholder garage_user
        let guid = Uuid::new_v4();
        let placeholder_metadata = json!({ "needs_setup": true });

        let garage_user: GarageUser = sqlx::query_as::<_, GarageUser>(
            r#"
    INSERT INTO garage_users
        (id, garage_id, username, password_hash, display_name, phone, email, role, metadata, is_active, created_at)
    VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, false, $10)
    RETURNING id, garage_id, username, password_hash, display_name, phone, email, role, metadata, is_active, created_at, updated_at, deleted_at
    "#
        )
            .bind(guid)
            .bind(gid)
            .bind(None::<String>)
            .bind(None::<String>)
            .bind(Some(format!("{} Admin", &new.name)))
            .bind(None::<String>)
            .bind(None::<String>)
            .bind("ADMIN")
            .bind(&placeholder_metadata)
            .bind(now)
            .fetch_one(&mut *tx)    // Single dereference
            .await
            .map_err(|e| eyre::eyre!(e))?;

        // commit transaction
        tx.commit().await.map_err(|e| eyre::eyre!(e))?;

        Ok((garage, garage_user))
    }

    pub async fn get_garage_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Garage>> {
        let rec = sqlx::query_as::<_, Garage>(
            r#"
            SELECT id, name, address, phone, email, metadata, created_at, updated_at
            FROM garages
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(rec)
    }

    pub async fn delete_garage_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Garage>> {
        let now = Utc::now();

        let rec = sqlx::query_as::<_, Garage>(
            r#"
            UPDATE garages
            SET deleted_at = $2, updated_at = $2
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, name, address, phone, email, metadata, created_at, updated_at, deleted_at
            "#,
        )
        .bind(id)
        .bind(now)
        .fetch_optional(pool)
        .await
        .map_err(|e| eyre::eyre!(e))?;

        Ok(rec)
    }
}
