use std::sync::Arc;

use async_trait::async_trait;

use qryvanta_core::AppResult;

/// Authentication event payload for security analytics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthEvent {
    /// Subject if available.
    pub subject: Option<String>,
    /// Stable event type identifier.
    pub event_type: String,
    /// Event outcome label (success or failure).
    pub outcome: String,
    /// Caller IP address if available.
    pub ip_address: Option<String>,
    /// Caller user-agent if available.
    pub user_agent: Option<String>,
}

/// Repository port for auth event persistence.
#[async_trait]
pub trait AuthEventRepository: Send + Sync {
    /// Appends an auth event entry.
    async fn append_event(&self, event: AuthEvent) -> AppResult<()>;
}

/// Application service for auth event recording.
#[derive(Clone)]
pub struct AuthEventService {
    repository: Arc<dyn AuthEventRepository>,
}

impl AuthEventService {
    /// Creates a service from a repository implementation.
    #[must_use]
    pub fn new(repository: Arc<dyn AuthEventRepository>) -> Self {
        Self { repository }
    }

    /// Persists an auth event.
    pub async fn record_event(&self, event: AuthEvent) -> AppResult<()> {
        self.repository.append_event(event).await
    }
}
