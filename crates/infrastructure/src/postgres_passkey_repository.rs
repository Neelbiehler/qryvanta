use qryvanta_core::{AppError, AppResult};
use sqlx::PgPool;

/// PostgreSQL-backed passkey credential persistence.
#[derive(Clone)]
pub struct PostgresPasskeyRepository {
    pool: PgPool,
}

impl PostgresPasskeyRepository {
    /// Creates a repository with the provided connection pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Lists all passkey credential payloads for a subject.
    pub async fn list_by_subject(&self, subject: &str) -> AppResult<Vec<String>> {
        let records = sqlx::query_scalar::<_, String>(
            r#"
            SELECT credential_json::text
            FROM passkey_credentials
            WHERE subject = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(subject)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to list passkeys: {error}")))?;

        Ok(records)
    }

    /// Persists a passkey credential payload for a subject.
    pub async fn insert_for_subject(&self, subject: &str, credential_json: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO passkey_credentials (subject, credential_json)
            VALUES ($1, $2::jsonb)
            "#,
        )
        .bind(subject)
        .bind(credential_json)
        .execute(&self.pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to insert passkey: {error}")))?;

        Ok(())
    }

    /// Replaces all passkeys for a subject with the supplied payloads.
    pub async fn replace_for_subject(
        &self,
        subject: &str,
        passkeys_json: &[String],
    ) -> AppResult<()> {
        let mut transaction =
            self.pool.begin().await.map_err(|error| {
                AppError::Internal(format!("failed to begin transaction: {error}"))
            })?;

        sqlx::query(
            r#"
            DELETE FROM passkey_credentials
            WHERE subject = $1
            "#,
        )
        .bind(subject)
        .execute(&mut *transaction)
        .await
        .map_err(|error| AppError::Internal(format!("failed to delete passkeys: {error}")))?;

        for credential_json in passkeys_json {
            sqlx::query(
                r#"
                INSERT INTO passkey_credentials (subject, credential_json)
                VALUES ($1, $2::jsonb)
                "#,
            )
            .bind(subject)
            .bind(credential_json)
            .execute(&mut *transaction)
            .await
            .map_err(|error| AppError::Internal(format!("failed to upsert passkey: {error}")))?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| AppError::Internal(format!("failed to commit transaction: {error}")))
    }
}
