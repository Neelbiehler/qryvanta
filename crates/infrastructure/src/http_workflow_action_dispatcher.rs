use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use qryvanta_application::{
    EmailService, WorkflowActionDispatchRequest, WorkflowActionDispatchType,
    WorkflowActionDispatcher,
};
use qryvanta_core::{AppError, AppResult, resolve_secret_reference};
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
            AppError::Validation("http_request payload must be an object".to_owned())
        })?;

        let url = payload.get("url").and_then(Value::as_str).ok_or_else(|| {
            AppError::Validation("http_request payload requires string field 'url'".to_owned())
        })?;
        let method = payload
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or("POST")
            .to_uppercase();
        let method = reqwest::Method::from_bytes(method.as_bytes()).map_err(|error| {
            AppError::Validation(format!(
                "http_request payload has invalid HTTP method: {error}"
            ))
        })?;

        let headers = payload
            .get("headers")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let resolved_secret_headers = resolve_secret_headers(
            payload.get("header_secret_refs"),
            "http_request",
            resolve_secret_reference,
        )
        .await?;
        let body = payload.get("body").cloned().unwrap_or(Value::Null);

        self.dispatch_with_retry(request, |client| {
            let trace_id = workflow_trace_id(request);
            let mut builder = client
                .request(method.clone(), url)
                .header("Idempotency-Key", request.idempotency_key.as_str())
                .header("X-Qryvanta-Workflow-Run", request.run_id.as_str())
                .header("X-Qryvanta-Workflow-Step", request.step_path.as_str())
                .header("X-Trace-Id", trace_id.as_str());

            for (key, value) in &headers {
                if let Some(header_value) = value.as_str() {
                    builder = builder.header(key, header_value);
                }
            }
            for (key, value) in &resolved_secret_headers {
                builder = builder.header(key, value);
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
        let payload = request
            .payload
            .as_object()
            .ok_or_else(|| AppError::Validation("webhook payload must be an object".to_owned()))?;

        let endpoint = payload
            .get("endpoint")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::Validation("webhook payload requires string field 'endpoint'".to_owned())
            })?;
        let event = payload
            .get("event")
            .and_then(Value::as_str)
            .unwrap_or("workflow.event");
        let headers = payload
            .get("headers")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let resolved_secret_headers = resolve_secret_headers(
            payload.get("header_secret_refs"),
            "webhook",
            resolve_secret_reference,
        )
        .await?;
        let event_payload = payload.get("payload").cloned().unwrap_or(Value::Null);

        self.dispatch_with_retry(request, |client| {
            let trace_id = workflow_trace_id(request);
            let mut builder = client
                .post(endpoint)
                .header("Idempotency-Key", request.idempotency_key.as_str())
                .header("X-Qryvanta-Workflow-Run", request.run_id.as_str())
                .header("X-Qryvanta-Workflow-Step", request.step_path.as_str())
                .header("X-Qryvanta-Webhook-Event", event)
                .header("X-Trace-Id", trace_id.as_str());

            for (key, value) in &headers {
                if let Some(header_value) = value.as_str() {
                    builder = builder.header(key, header_value);
                }
            }
            for (key, value) in &resolved_secret_headers {
                builder = builder.header(key, value);
            }

            builder.json(&serde_json::json!({
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
            AppError::Validation("send_email payload must be an object".to_owned())
        })?;

        let to = payload.get("to").and_then(Value::as_str).ok_or_else(|| {
            AppError::Validation("send_email payload requires string field 'to'".to_owned())
        })?;
        let subject = payload
            .get("subject")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AppError::Validation(
                    "send_email payload requires string field 'subject'".to_owned(),
                )
            })?;
        let body = payload.get("body").and_then(Value::as_str).ok_or_else(|| {
            AppError::Validation("send_email payload requires string field 'body'".to_owned())
        })?;
        let html_body = payload.get("html_body").and_then(Value::as_str);

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

fn workflow_trace_id(request: &WorkflowActionDispatchRequest) -> String {
    format!("workflow-{}-{}", request.run_id, request.step_path)
}

async fn resolve_secret_headers<F>(
    header_secret_refs: Option<&Value>,
    step_type: &str,
    resolver: F,
) -> AppResult<Vec<(String, String)>>
where
    F: Fn(&str) -> AppResult<String> + Send + Sync + Copy + 'static,
{
    let Some(header_secret_refs) = header_secret_refs.and_then(Value::as_object) else {
        return Ok(Vec::new());
    };

    let refs = header_secret_refs
        .iter()
        .map(|(key, value)| {
            let reference = value.as_str().ok_or_else(|| {
                AppError::Validation(format!(
                    "{step_type} payload field 'header_secret_refs.{key}' must be a string"
                ))
            })?;
            Ok((key.clone(), reference.to_owned()))
        })
        .collect::<AppResult<Vec<(String, String)>>>()?;

    tokio::task::spawn_blocking(move || {
        refs.into_iter()
            .map(|(key, reference)| {
                resolve_workflow_header_secret_reference(reference.as_str(), resolver)
                    .map(|value| (key, value))
            })
            .collect::<AppResult<Vec<(String, String)>>>()
    })
    .await
    .map_err(|error| AppError::Internal(format!("failed to resolve secret headers: {error}")))?
}

fn resolve_workflow_header_secret_reference<F>(reference: &str, resolver: F) -> AppResult<String>
where
    F: Fn(&str) -> AppResult<String> + Copy,
{
    if let Some(inner_reference) = reference.strip_prefix("bearer+") {
        return resolver(inner_reference).map(|value| format!("Bearer {value}"));
    }

    if let Some(inner_reference) = reference.strip_prefix("basic+") {
        return resolver(inner_reference).map(|value| format!("Basic {value}"));
    }

    resolver(reference)
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

#[cfg(test)]
mod tests {
    use super::{resolve_secret_headers, resolve_workflow_header_secret_reference};
    use qryvanta_core::{AppError, AppResult};
    use serde_json::json;

    #[tokio::test]
    async fn resolves_header_secret_refs_with_injected_resolver() {
        let resolved = resolve_secret_headers(
            Some(&json!({
                "authorization": "op://vault/item/password",
                "x-api-key": "aws-sm://prod/api-key"
            })),
            "http_request",
            |reference| Ok(format!("resolved:{reference}")),
        )
        .await
        .unwrap_or_else(|_| unreachable!());

        assert_eq!(
            resolved,
            vec![
                (
                    "authorization".to_owned(),
                    "resolved:op://vault/item/password".to_owned()
                ),
                (
                    "x-api-key".to_owned(),
                    "resolved:aws-sm://prod/api-key".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn resolves_formatted_authorization_secret_refs() {
        let bearer =
            resolve_workflow_header_secret_reference("bearer+op://vault/item/token", |reference| {
                Ok(format!("resolved:{reference}"))
            })
            .unwrap_or_else(|_| unreachable!());
        let basic = resolve_workflow_header_secret_reference(
            "basic+aws-sm://prod/basic-creds",
            |reference| Ok(format!("resolved:{reference}")),
        )
        .unwrap_or_else(|_| unreachable!());

        assert_eq!(bearer, "Bearer resolved:op://vault/item/token");
        assert_eq!(basic, "Basic resolved:aws-sm://prod/basic-creds");
    }

    #[tokio::test]
    async fn propagates_secret_resolution_errors() {
        let result = resolve_secret_headers(
            Some(&json!({
                "authorization": "op://vault/item/password"
            })),
            "webhook",
            |_reference| -> AppResult<String> {
                Err(AppError::Validation("resolver failed".to_owned()))
            },
        )
        .await;

        assert!(
            matches!(result, Err(AppError::Validation(message)) if message == "resolver failed")
        );
    }
}
