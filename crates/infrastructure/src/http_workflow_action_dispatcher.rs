use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use qryvanta_application::{
    EmailService, WorkflowActionDispatchRequest, WorkflowActionDispatchType,
    WorkflowActionDispatcher,
};
use qryvanta_core::{AppError, AppResult};
use serde_json::Value;

/// HTTP-based implementation for workflow external action dispatch.
pub struct HttpWorkflowActionDispatcher {
    http_client: reqwest::Client,
    email_service: Arc<dyn EmailService>,
    max_attempts: u8,
    retry_backoff_ms: u64,
}

impl HttpWorkflowActionDispatcher {
    /// Creates a new workflow action dispatcher.
    #[must_use]
    pub fn new(
        http_client: reqwest::Client,
        email_service: Arc<dyn EmailService>,
        max_attempts: u8,
        retry_backoff_ms: u64,
    ) -> Self {
        Self {
            http_client,
            email_service,
            max_attempts: max_attempts.max(1),
            retry_backoff_ms: retry_backoff_ms.max(50),
        }
    }

    async fn dispatch_http_request(
        &self,
        request: &WorkflowActionDispatchRequest,
    ) -> AppResult<()> {
        let payload = request.payload.as_object().ok_or_else(|| {
            AppError::Validation("integration_http_request payload must be an object".to_owned())
        })?;

        let url = payload.get("url").and_then(Value::as_str).ok_or_else(|| {
            AppError::Validation(
                "integration_http_request payload requires string field 'url'".to_owned(),
            )
        })?;
        let method = payload
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("POST")
            .to_uppercase();
        let method = reqwest::Method::from_bytes(method.as_bytes()).map_err(|error| {
            AppError::Validation(format!(
                "integration_http_request payload has invalid HTTP method: {error}"
            ))
        })?;

        let headers = payload
            .get("headers")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let body = payload.get("body").cloned().unwrap_or(Value::Null);

        self.dispatch_with_retry(request, |client| {
            let mut builder = client
                .request(method.clone(), url)
                .header("Idempotency-Key", request.idempotency_key.as_str())
                .header("X-Qryvanta-Workflow-Run", request.run_id.as_str())
                .header("X-Qryvanta-Workflow-Step", request.step_path.as_str());

            for (key, value) in &headers {
                if let Some(header_value) = value.as_str() {
                    builder = builder.header(key, header_value);
                }
            }

            if body.is_null() {
                builder
            } else {
                builder.json(&body)
            }
        })
        .await
    }

    async fn dispatch_webhook(&self, request: &WorkflowActionDispatchRequest) -> AppResult<()> {
        let payload = request.payload.as_object().ok_or_else(|| {
            AppError::Validation("webhook_dispatch payload must be an object".to_owned())
        })?;

        let endpoint = payload
            .get("endpoint")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::Validation(
                    "webhook_dispatch payload requires string field 'endpoint'".to_owned(),
                )
            })?;
        let event = payload
            .get("event")
            .and_then(Value::as_str)
            .unwrap_or("workflow.event");
        let event_payload = payload.get("payload").cloned().unwrap_or(Value::Null);

        self.dispatch_with_retry(request, |client| {
            client
                .post(endpoint)
                .header("Idempotency-Key", request.idempotency_key.as_str())
                .header("X-Qryvanta-Workflow-Run", request.run_id.as_str())
                .header("X-Qryvanta-Webhook-Event", event)
                .json(&serde_json::json!({
                    "event": event,
                    "payload": event_payload,
                    "run_id": request.run_id,
                    "step_path": request.step_path,
                }))
        })
        .await
    }

    async fn dispatch_email(&self, request: &WorkflowActionDispatchRequest) -> AppResult<()> {
        let payload = request.payload.as_object().ok_or_else(|| {
            AppError::Validation("email_outbox payload must be an object".to_owned())
        })?;

        let to = payload.get("to").and_then(Value::as_str).ok_or_else(|| {
            AppError::Validation("email_outbox payload requires string field 'to'".to_owned())
        })?;
        let subject = payload
            .get("subject")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::Validation(
                    "email_outbox payload requires string field 'subject'".to_owned(),
                )
            })?;
        let body = payload.get("body").and_then(Value::as_str).ok_or_else(|| {
            AppError::Validation("email_outbox payload requires string field 'body'".to_owned())
        })?;
        let html_body = payload.get("html").and_then(Value::as_str);

        self.email_service
            .send_email(to, subject, body, html_body)
            .await
    }

    async fn dispatch_with_retry<F>(
        &self,
        request: &WorkflowActionDispatchRequest,
        mut build: F,
    ) -> AppResult<()>
    where
        F: FnMut(&reqwest::Client) -> reqwest::RequestBuilder,
    {
        let mut attempt = 0_u8;
        let mut last_error: Option<String> = None;

        while attempt < self.max_attempts {
            attempt = attempt.saturating_add(1);
            let response = build(&self.http_client).send().await;

            match response {
                Ok(response) if response.status().is_success() => return Ok(()),
                Ok(response)
                    if response.status().is_server_error()
                        || response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS =>
                {
                    last_error = Some(format!(
                        "transient HTTP status {} for workflow dispatch '{}'",
                        response.status(),
                        request.idempotency_key
                    ));
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "<response body unavailable>".to_owned());
                    return Err(AppError::Validation(format!(
                        "workflow external dispatch failed with status {status}: {body}"
                    )));
                }
                Err(error) => {
                    last_error = Some(format!(
                        "workflow external dispatch transport error: {error}"
                    ));
                }
            }

            if attempt < self.max_attempts {
                let delay = self.retry_backoff_ms.saturating_mul(u64::from(attempt));
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }

        Err(AppError::Internal(last_error.unwrap_or_else(|| {
            "workflow external dispatch exhausted retries".to_owned()
        })))
    }
}

#[async_trait]
impl WorkflowActionDispatcher for HttpWorkflowActionDispatcher {
    async fn dispatch_action(&self, request: WorkflowActionDispatchRequest) -> AppResult<()> {
        match request.dispatch_type {
            WorkflowActionDispatchType::HttpRequest => self.dispatch_http_request(&request).await,
            WorkflowActionDispatchType::Webhook => self.dispatch_webhook(&request).await,
            WorkflowActionDispatchType::Email => self.dispatch_email(&request).await,
        }
    }
}
