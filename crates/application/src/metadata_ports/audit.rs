use async_trait::async_trait;
use qryvanta_core::{AppResult, TenantId};
use qryvanta_domain::AuditAction;

/// Immutable audit event payload emitted by application services.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    /// Tenant scope for the event.
    pub tenant_id: TenantId,
    /// Subject that performed the action.
    pub subject: String,
    /// Stable audit action identifier.
    pub action: AuditAction,
    /// Resource type label.
    pub resource_type: String,
    /// Resource identifier.
    pub resource_id: String,
    /// Optional audit detail payload.
    pub detail: Option<String>,
}

/// Port for persisting append-only audit events.
#[async_trait]
pub trait AuditRepository: Send + Sync {
    /// Persists one audit event.
    async fn append_event(&self, event: AuditEvent) -> AppResult<()>;
}
