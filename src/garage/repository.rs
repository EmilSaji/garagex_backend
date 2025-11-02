use eyre::Result;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::models::{
    GarageUser,
    JobCreateRequest,
    JobCreatedResponse,
    JobDetailsResponse,
    JobListItem,
    JobPartItem,
    JobStatusHistoryItem,
    JobStatusUpdateResponse,
    JobStatusUpdateRequest,
    JobPartCreateItem,
    JobPartUpdateRequest,
};

pub struct GarageRepo;

impl GarageRepo {
    pub async fn find_user_by_username(pool: &PgPool, username: &str) -> Result<GarageUser> {
        let rec = sqlx::query_as::<_, GarageUser>(
            r#"
            SELECT 
                id,
                garage_id,
                username,
                password_hash,
                display_name,
                phone,
                email,
                role,
                is_active
            FROM garage_users
            WHERE username = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(username)
        .fetch_one(pool)
        .await?;
        Ok(rec)
    }

    pub async fn list_jobs_for_garage_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<JobListItem>> {
        let rows = sqlx::query_as::<_, JobListItem>(
            r#"
            SELECT 
                j.id AS job_id,
                v.vehicle_number AS vehicle_number,
                c.name AS owner_name,
                j.estimated_delivery_date AS estimated_delivery_date,
                j.estimated_time AS estimated_time,
                (j.status)::text AS status
            FROM jobs j
            LEFT JOIN vehicles v ON v.id = j.vehicle_id
            LEFT JOIN customers c ON c.id = v.customer_id
            WHERE j.garage_id = (
                SELECT gu.garage_id FROM garage_users gu
                WHERE gu.id = $1 AND gu.deleted_at IS NULL
            )
              AND j.deleted_at IS NULL
            ORDER BY j.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(rows)
    }

    pub async fn create_job_with_entities(
        pool: &PgPool,
        garage_user_id: Uuid,
        req: &JobCreateRequest,
    ) -> Result<JobCreatedResponse> {
        let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

        // Find garage_id from garage_users
        let garage_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT garage_id FROM garage_users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(garage_user_id)
        .fetch_optional(&mut *tx)
        .await?;

        let garage_id = match garage_id {
            Some(g) => g,
            None => return Err(eyre::eyre!("garage user not found or inactive")),
        };

        // Upsert customer by phone
        let customer_row = sqlx::query_as::<_, (Uuid, Option<String>)>(
            r#"
            INSERT INTO customers (phone, name)
            VALUES ($1, $2)
            ON CONFLICT (phone)
            DO UPDATE SET name = COALESCE(EXCLUDED.name, customers.name)
            RETURNING id, name
            "#,
        )
        .bind(&req.phone)
        .bind(req.customer_name.as_ref())
        .fetch_one(&mut *tx)
        .await?;
        let (customer_id, customer_name) = customer_row;

        // Upsert vehicle by (customer_id, vehicle_number)
        let vehicle_row = sqlx::query_as::<_, (Uuid, String)>(
            r#"
            INSERT INTO vehicles (customer_id, vehicle_number, make, model)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (customer_id, vehicle_number)
            DO UPDATE SET 
                make = COALESCE(EXCLUDED.make, vehicles.make),
                model = COALESCE(EXCLUDED.model, vehicles.model)
            RETURNING id, vehicle_number
            "#,
        )
        .bind(customer_id)
        .bind(&req.vehicle_number)
        .bind(req.vehicle_make.as_ref())
        .bind(req.vehicle_model.as_ref())
        .fetch_one(&mut *tx)
        .await?;
        let (vehicle_id, vehicle_number) = vehicle_row;

        // Generate a job identifier (simple UUID-based)
        let job_identifier = format!("JOB-{}", Uuid::new_v4().to_string());

        // Insert job
        let job_row = sqlx::query_as::<_, (Uuid, String, Option<chrono::NaiveDate>, Option<String>, String)>(
            r#"
            INSERT INTO jobs (
                job_identifier, garage_id, vehicle_id, customer_phone, customer_name,
                complaint, estimated_delivery_date, estimated_time
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, job_identifier, estimated_delivery_date, estimated_time, (status)::text
            "#,
        )
        .bind(&job_identifier)
        .bind(garage_id)
        .bind(vehicle_id)
        .bind(&req.phone)
        .bind(req.customer_name.as_ref().or(customer_name.as_ref()))
        .bind(req.complaint.as_ref())
        .bind(req.estimated_delivery_date)
        .bind(req.estimated_time.as_ref())
        .fetch_one(&mut *tx)
        .await?;
        let (job_id, job_identifier, est_date, est_time, status) = job_row;

        // Add initial status history entry
        sqlx::query(
            r#"
            INSERT INTO job_status_history (job_id, from_status, to_status, note)
            VALUES ($1, NULL, $2::job_status, $3)
            "#,
        )
        .bind(job_id)
        .bind(&status)
        .bind("Job created")
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(JobCreatedResponse {
            job_id,
            job_identifier,
            vehicle_id,
            customer_id,
            vehicle_number,
            owner_name: customer_name,
            estimated_delivery_date: est_date,
            estimated_time: est_time,
            status,
        })
    }

    pub async fn get_job_details(pool: &PgPool, job_id: Uuid) -> Result<JobDetailsResponse> {
        // Header details: job + vehicle + customer
        let (jid, status, remarks, vehicle_number, vehicle_make, vehicle_model, owner_name) =
            sqlx::query_as::<_, (
                Uuid,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
            )>(
                r#"
                SELECT 
                    j.id,
                    (j.status)::text AS status,
                    j.remarks,
                    v.vehicle_number,
                    v.make,
                    v.model,
                    c.name AS owner_name
                FROM jobs j
                LEFT JOIN vehicles v ON v.id = j.vehicle_id
                LEFT JOIN customers c ON c.id = v.customer_id
                WHERE j.id = $1 AND j.deleted_at IS NULL
                "#,
            )
            .bind(job_id)
            .fetch_one(pool)
            .await?;

        // Parts used in the job
        let parts: Vec<JobPartItem> = sqlx::query_as::<_, JobPartItem>(
            r#"
            SELECT 
                id,
                name,
                quantity,
                unit_price::float8 as unit_price,
                tax_percent::float8 as tax_percent
            FROM job_parts
            WHERE job_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(pool)
        .await?;

        // Status history
        let status_history: Vec<JobStatusHistoryItem> = sqlx::query_as::<_, JobStatusHistoryItem>(
            r#"
            SELECT 
                id,
                (from_status)::text AS from_status,
                (to_status)::text AS to_status,
                note,
                created_at
            FROM job_status_history
            WHERE job_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(pool)
        .await?;

        Ok(JobDetailsResponse {
            job_id: jid,
            status,
            remarks,
            vehicle_number,
            vehicle_make,
            vehicle_model,
            owner_name,
            parts,
            status_history,
        })
    }

    pub async fn update_job_status(
        pool: &PgPool,
        job_id: Uuid,
        body: &JobStatusUpdateRequest,
    ) -> Result<JobStatusUpdateResponse> {
        let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

        // Fetch current status
        let from_status: Option<String> = sqlx::query_scalar(
            r#"
            SELECT (status)::text FROM jobs WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await?;

        if from_status.is_none() {
            return Err(eyre::eyre!("job not found"));
        }

        // Update job status and optionally remarks
        sqlx::query(
            r#"
            UPDATE jobs 
            SET status = $2::job_status,
                remarks = COALESCE($3, remarks),
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .bind(&body.to_status)
        .bind(body.remarks.as_ref())
        .execute(&mut *tx)
        .await?;

        // Insert status history row
        sqlx::query(
            r#"
            INSERT INTO job_status_history (job_id, from_status, to_status, note)
            VALUES ($1, $2::job_status, $3::job_status, $4)
            "#,
        )
        .bind(job_id)
        .bind(from_status.as_deref())
        .bind(&body.to_status)
        .bind(body.note.as_deref())
        .execute(&mut *tx)
        .await?;

        // Parts are no longer handled here; separate endpoints manage parts.

        // Fetch full status history after update
        let status_history: Vec<JobStatusHistoryItem> = sqlx::query_as::<_, JobStatusHistoryItem>(
            r#"
            SELECT 
                id,
                (from_status)::text AS from_status,
                (to_status)::text AS to_status,
                note,
                created_at
            FROM job_status_history
            WHERE job_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(JobStatusUpdateResponse { status_history })
    }

    pub async fn add_job_parts(
        pool: &PgPool,
        job_id: Uuid,
        parts: &Vec<JobPartCreateItem>,
    ) -> Result<Vec<JobPartItem>> {
        let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

        for p in parts {
            sqlx::query(
                r#"
                INSERT INTO job_parts (job_id, name, quantity, unit_price, tax_percent)
                VALUES ($1, $2, COALESCE($3, 1), $4, $5)
                "#,
            )
            .bind(job_id)
            .bind(&p.name)
            .bind(p.quantity)
            .bind(p.unit_price)
            .bind(p.tax_percent)
            .execute(&mut *tx)
            .await?;
        }

        let parts_out: Vec<JobPartItem> = sqlx::query_as::<_, JobPartItem>(
            r#"
            SELECT id, name, quantity, unit_price::float8 as unit_price, tax_percent::float8 as tax_percent
            FROM job_parts
            WHERE job_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(parts_out)
    }

    pub async fn update_job_part(
        pool: &PgPool,
        job_id: Uuid,
        part_id: Uuid,
        req: &JobPartUpdateRequest,
    ) -> Result<JobPartItem> {
        let rec = sqlx::query_as::<_, JobPartItem>(
            r#"
            UPDATE job_parts
            SET 
                name = COALESCE($3, name),
                quantity = COALESCE($4, quantity),
                unit_price = COALESCE($5, unit_price),
                tax_percent = COALESCE($6, tax_percent)
            WHERE id = $1 AND job_id = $2
            RETURNING id, name, quantity, unit_price::float8 as unit_price, tax_percent::float8 as tax_percent
            "#,
        )
        .bind(part_id)
        .bind(job_id)
        .bind(req.name.as_ref())
        .bind(req.quantity)
        .bind(req.unit_price)
        .bind(req.tax_percent)
        .fetch_optional(pool)
        .await?;

        match rec {
            Some(row) => Ok(row),
            None => Err(eyre::eyre!("part not found for this job")),
        }
    }

    pub async fn remove_job_part(
        pool: &PgPool,
        job_id: Uuid,
        part_id: Uuid,
    ) -> Result<Vec<JobPartItem>> {
        // Delete the part ensuring it belongs to the job
        let result = sqlx::query(
            r#"
            DELETE FROM job_parts 
            WHERE id = $1 AND job_id = $2
            "#,
        )
        .bind(part_id)
        .bind(job_id)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(eyre::eyre!("part not found for this job"));
        }

        // Return remaining parts for the job
        let parts: Vec<JobPartItem> = sqlx::query_as::<_, JobPartItem>(
            r#"
            SELECT 
                id,
                name,
                quantity,
                unit_price::float8 as unit_price,
                tax_percent::float8 as tax_percent
            FROM job_parts
            WHERE job_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(job_id)
        .fetch_all(pool)
        .await?;

        Ok(parts)
    }
}
