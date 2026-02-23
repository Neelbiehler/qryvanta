use async_trait::async_trait;
use chrono::Utc;
use redis::AsyncCommands;
use tower_sessions::SessionStore;
use tower_sessions::session::{Id, Record};
use tower_sessions::session_store::{self, Error as SessionStoreError};

#[derive(Debug, Clone)]
pub struct RedisSessionStore {
    client: redis::Client,
    key_prefix: String,
}

impl RedisSessionStore {
    #[must_use]
    pub fn new(client: redis::Client, key_prefix: impl Into<String>) -> Self {
        Self {
            client,
            key_prefix: key_prefix.into(),
        }
    }

    fn key_for(&self, session_id: &Id) -> String {
        format!("{}:{}", self.key_prefix, session_id)
    }
}

#[async_trait]
impl SessionStore for RedisSessionStore {
    async fn save(&self, session_record: &Record) -> session_store::Result<()> {
        let key = self.key_for(&session_record.id);
        let ttl_seconds = session_record.expiry_date.unix_timestamp() - Utc::now().timestamp();

        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| SessionStoreError::Backend(error.to_string()))?;

        if ttl_seconds <= 0 {
            connection
                .del::<_, i64>(key)
                .await
                .map_err(|error| SessionStoreError::Backend(error.to_string()))?;
            return Ok(());
        }

        let encoded_record = serde_json::to_string(session_record)
            .map_err(|error| SessionStoreError::Encode(error.to_string()))?;

        let ttl_seconds = u64::try_from(ttl_seconds)
            .map_err(|error| SessionStoreError::Backend(error.to_string()))?;

        connection
            .set_ex::<_, _, ()>(key, encoded_record, ttl_seconds)
            .await
            .map_err(|error| SessionStoreError::Backend(error.to_string()))
    }

    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        let key = self.key_for(session_id);
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| SessionStoreError::Backend(error.to_string()))?;

        let encoded_record: Option<String> = connection
            .get(key)
            .await
            .map_err(|error| SessionStoreError::Backend(error.to_string()))?;

        encoded_record
            .as_deref()
            .map(|value| {
                serde_json::from_str::<Record>(value)
                    .map_err(|error| SessionStoreError::Decode(error.to_string()))
            })
            .transpose()
    }

    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        let key = self.key_for(session_id);
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| SessionStoreError::Backend(error.to_string()))?;

        connection
            .del::<_, i64>(key)
            .await
            .map_err(|error| SessionStoreError::Backend(error.to_string()))?;

        Ok(())
    }
}
