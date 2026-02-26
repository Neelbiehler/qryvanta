use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use qryvanta_core::AppResult;
use qryvanta_domain::AuthTokenType;

use super::{AuthTokenRecord, AuthTokenRepository, AuthTokenService, EmailService};

#[derive(Default)]
struct TestTokenRepo {
    created: Mutex<Vec<(String, AuthTokenType, Option<serde_json::Value>)>>,
}

#[async_trait]
impl AuthTokenRepository for TestTokenRepo {
    async fn create_token(
        &self,
        _user_id: Option<qryvanta_domain::UserId>,
        email: &str,
        _token_hash: &str,
        token_type: AuthTokenType,
        _expires_at: chrono::DateTime<chrono::Utc>,
        metadata: Option<&serde_json::Value>,
    ) -> AppResult<uuid::Uuid> {
        self.created
            .lock()
            .map_err(|error| {
                qryvanta_core::AppError::Internal(format!("failed to lock repo state: {error}"))
            })?
            .push((email.to_owned(), token_type, metadata.cloned()));
        Ok(uuid::Uuid::new_v4())
    }

    async fn consume_valid_token(
        &self,
        _token_hash: &str,
        _token_type: AuthTokenType,
    ) -> AppResult<Option<AuthTokenRecord>> {
        Ok(None)
    }

    async fn invalidate_tokens_for_user(
        &self,
        _user_id: qryvanta_domain::UserId,
        _token_type: AuthTokenType,
    ) -> AppResult<()> {
        Ok(())
    }

    async fn count_recent_tokens(
        &self,
        _email: &str,
        _token_type: AuthTokenType,
        _since: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<i64> {
        Ok(0)
    }
}

#[derive(Default)]
struct TestEmailService {
    sent: Mutex<Vec<(String, String)>>,
}

#[async_trait]
impl EmailService for TestEmailService {
    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        _text_body: &str,
        _html_body: Option<&str>,
    ) -> AppResult<()> {
        self.sent
            .lock()
            .map_err(|error| {
                qryvanta_core::AppError::Internal(format!(
                    "failed to lock email service state: {error}"
                ))
            })?
            .push((to.to_owned(), subject.to_owned()));
        Ok(())
    }
}

#[tokio::test]
async fn send_invite_persists_invite_token_and_sends_email() {
    let repo = Arc::new(TestTokenRepo::default());
    let email = Arc::new(TestEmailService::default());

    let service = AuthTokenService::new(
        repo.clone(),
        email.clone(),
        "http://localhost:3000".to_owned(),
    );

    let metadata = serde_json::json!({"tenant_id": "tenant-1", "invited_by": "alice"});
    let result = service
        .send_invite("new.user@example.com", "Alice", "Acme Workspace", &metadata)
        .await;

    assert!(result.is_ok());

    let created = repo
        .created
        .lock()
        .ok()
        .map(|guard| guard.len())
        .unwrap_or(0);
    assert_eq!(created, 1);

    let sent = email.sent.lock().ok().map(|guard| guard.len()).unwrap_or(0);
    assert_eq!(sent, 1);
}
