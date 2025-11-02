use chrono::Utc;
use eyre::Result;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction}; // <-- Executor is needed
use uuid::Uuid;

use crate::admin::models::AdminUser;
use crate::admin::models::{
    Garage, GarageUser, ManageCredentials, NewGarage, SingleGarage, UpdateGarage,
};

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
        ($1, $2, $3, $4, $5, $6, $7, $8, $9, true, $10)
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

    pub async fn get_garage_by_id(pool: &PgPool, id: Uuid) -> Result<Option<SingleGarage>> {
        let rec = sqlx::query_as::<_, SingleGarage>(
            r#"
        SELECT
            g.id,
            g.name,
            g.address,
            g.phone,
            g.email,
            g.metadata,
            g.created_at,
            g.updated_at,
            (
                SELECT gu.username
                FROM garage_users gu
                WHERE gu.garage_id = g.id
                  AND gu.role = 'ADMIN'
                  AND gu.deleted_at IS NULL
                ORDER BY gu.created_at DESC
                LIMIT 1
            ) AS username
        FROM garages g
        WHERE g.id = $1
          AND g.deleted_at IS NULL
        "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(rec)
    }

    pub async fn delete_garage_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Garage>> {
        let now = Utc::now();

        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(|e| eyre::eyre!(e))?;

        sqlx::query(
            r#"
        UPDATE garage_users
        SET deleted_at = $2, updated_at = $2
        WHERE garage_id = $1 AND deleted_at IS NULL
        "#,
        )
        .bind(id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| eyre::eyre!(e))?;

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
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| eyre::eyre!(e))?;

        // Commit transaction
        tx.commit().await.map_err(|e| eyre::eyre!(e))?;

        Ok(rec)
    }

    pub async fn update_garage_by_id(
        pool: &PgPool,
        id: Uuid,
        update: &UpdateGarage,
    ) -> Result<Option<Garage>> {
        let now = Utc::now();

        let rec = sqlx::query_as::<_, Garage>(
            r#"
        UPDATE garages
        SET 
            name = COALESCE($2, name),
            address = COALESCE($3, address),
            phone = COALESCE($4, phone),
            email = COALESCE($5, email),
            metadata = COALESCE($6, metadata),
            updated_at = $7
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING 
            id, name, address, phone, email, metadata, created_at, updated_at
        "#,
        )
        .bind(id)
        .bind(update.name.as_deref())
        .bind(update.address.as_deref())
        .bind(update.phone.as_deref())
        .bind(update.email.as_deref())
        .bind(update.metadata.as_ref())
        .bind(now)
        .fetch_optional(pool)
        .await
        .map_err(|e| eyre::eyre!(e))?;

        Ok(rec)
    }

    pub async fn manage_garage_credentials(
        pool: &PgPool,
        garage_id: Uuid,
        creds: &ManageCredentials,
    ) -> Result<Option<GarageUser>> {
        let now = Utc::now();

        let rec = sqlx::query_as::<_, GarageUser>(
            r#"
        UPDATE garage_users
        SET
            username = COALESCE($2, username),
            password_hash = COALESCE($3, password_hash),
            updated_at = $4
        WHERE garage_id = $1
          AND role = 'ADMIN'
          AND deleted_at IS NULL
        RETURNING
            id,
            garage_id,
            username,
            display_name,
            password_hash,
            phone,
            email,
            role,
            metadata,
            is_active,
            created_at,
            updated_at,
            deleted_at
        "#,
        )
        .bind(garage_id)
        .bind(creds.username.as_deref()) // Option<&str> -> maps to SQL NULL or string
        .bind(creds.password_hash.as_deref()) // NOTE: ideally hash password before storing
        .bind(now)
        .fetch_optional(pool)
        .await
        .map_err(|e| eyre::eyre!(e))?;

        Ok(rec)
    }
}
