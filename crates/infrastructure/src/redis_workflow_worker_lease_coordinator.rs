//! Redis-backed distributed lease coordinator for workflow workers.

use async_trait::async_trait;
use qryvanta_application::{WorkflowWorkerLease, WorkflowWorkerLeaseCoordinator};
use qryvanta_core::{AppError, AppResult};
use redis::{AsyncCommands, Script};

const RELEASE_LEASE_SCRIPT: &str = r#"
if redis.call('GET', KEYS[1]) == ARGV[1] then
  return redis.call('DEL', KEYS[1])
else
  return 0
end
"#;

const RENEW_LEASE_SCRIPT: &str = r#"
if redis.call('GET', KEYS[1]) == ARGV[1] then
  return redis.call('EXPIRE', KEYS[1], ARGV[2])
else
  return 0
end
"#;

/// Redis implementation of workflow worker lease coordination.
#[derive(Clone)]
pub struct RedisWorkflowWorkerLeaseCoordinator {
    client: redis::Client,
    key_prefix: String,
}

impl RedisWorkflowWorkerLeaseCoordinator {
    /// Creates one coordinator adapter.
    #[must_use]
    pub fn new(client: redis::Client, key_prefix: impl Into<String>) -> Self {
        Self {
            client,
            key_prefix: key_prefix.into(),
        }
    }

    fn key_for(&self, scope_key: &str) -> String {
        format!("{}:{scope_key}", self.key_prefix)
    }
}

#[async_trait]
impl WorkflowWorkerLeaseCoordinator for RedisWorkflowWorkerLeaseCoordinator {
    async fn try_acquire_lease(
        &self,
        scope_key: &str,
        holder_id: &str,
        lease_seconds: u32,
    ) -> AppResult<Option<WorkflowWorkerLease>> {
        if scope_key.trim().is_empty() {
            return Err(AppError::Validation(
                "workflow worker lease scope_key must not be empty".to_owned(),
            ));
        }

        if holder_id.trim().is_empty() {
            return Err(AppError::Validation(
                "workflow worker lease holder_id must not be empty".to_owned(),
            ));
        }

        if lease_seconds == 0 {
            return Err(AppError::Validation(
                "workflow worker lease_seconds must be greater than zero".to_owned(),
            ));
        }

        let key = self.key_for(scope_key);
        let token = format!("{holder_id}:{}", uuid::Uuid::new_v4());

        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| AppError::Internal(format!("failed to connect to redis: {error}")))?;

        let acquired: bool = connection
            .set_nx(key.as_str(), token.as_str())
            .await
            .map_err(|error| {
                AppError::Internal(format!("failed to acquire worker lease: {error}"))
            })?;

        if !acquired {
            return Ok(None);
        }

        connection
            .expire::<_, ()>(key.as_str(), i64::from(lease_seconds))
            .await
            .map_err(|error| {
                AppError::Internal(format!("failed to set worker lease ttl: {error}"))
            })?;

        Ok(Some(WorkflowWorkerLease {
            scope_key: scope_key.to_owned(),
            token,
            holder_id: holder_id.to_owned(),
        }))
    }

    async fn release_lease(&self, lease: &WorkflowWorkerLease) -> AppResult<()> {
        let key = self.key_for(lease.scope_key.as_str());
        let script = Script::new(RELEASE_LEASE_SCRIPT);

        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| AppError::Internal(format!("failed to connect to redis: {error}")))?;

        script
            .key(key)
            .arg(lease.token.as_str())
            .invoke_async::<i32>(&mut connection)
            .await
            .map_err(|error| {
                AppError::Internal(format!("failed to release worker lease: {error}"))
            })?;

        Ok(())
    }

    async fn renew_lease(
        &self,
        lease: &WorkflowWorkerLease,
        lease_seconds: u32,
    ) -> AppResult<bool> {
        if lease_seconds == 0 {
            return Err(AppError::Validation(
                "workflow worker lease_seconds must be greater than zero".to_owned(),
            ));
        }

        let key = self.key_for(lease.scope_key.as_str());
        let script = Script::new(RENEW_LEASE_SCRIPT);

        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| AppError::Internal(format!("failed to connect to redis: {error}")))?;

        let renewed = script
            .key(key)
            .arg(lease.token.as_str())
            .arg(i64::from(lease_seconds))
            .invoke_async::<i32>(&mut connection)
            .await
            .map_err(|error| {
                AppError::Internal(format!("failed to renew worker lease: {error}"))
            })?;

        Ok(renewed > 0)
    }
}
