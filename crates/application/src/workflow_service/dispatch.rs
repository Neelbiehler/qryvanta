use super::*;
use crate::WorkflowScheduleTickDrainResult;
use chrono::{Datelike, Timelike};

const SCHEDULE_CLOCK_SKEW_TOLERANCE_SECONDS: i64 = 300;

struct ScheduleTickSlot {
    slot_key: String,
    tick_at_utc: chrono::DateTime<Utc>,
}

impl WorkflowService {
    pub(super) async fn dispatch_trigger(
        &self,
        actor: &UserIdentity,
        trigger: WorkflowTrigger,
        mut payload: Value,
    ) -> AppResult<usize> {
        let workflows = self
            .repository
            .list_enabled_workflows_for_trigger(actor.tenant_id(), &trigger)
            .await?;

        if workflows.is_empty() {
            return Ok(0);
        }

        let workflow_actor = UserIdentity::new(
            "workflow-runtime",
            "workflow-runtime",
            None,
            actor.tenant_id(),
        );

        if let Some(payload_object) = payload.as_object_mut() {
            payload_object
                .entry("triggered_by".to_owned())
                .or_insert_with(|| Value::String(actor.subject().to_owned()));
        }

        let mut executed = 0;
        for workflow in workflows {
            let result = match self.execution_mode {
                WorkflowExecutionMode::Inline => {
                    self.execute_workflow_definition(&workflow_actor, &workflow, payload.clone())
                        .await
                }
                WorkflowExecutionMode::Queued => {
                    self.enqueue_workflow_definition(&workflow_actor, &workflow, payload.clone())
                        .await
                }
            };

            if result.is_ok() {
                executed += 1;
            }
        }

        Ok(executed)
    }

    /// Executes a workflow by logical name using manual trigger context.
    pub async fn execute_workflow(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
        mut trigger_payload: Value,
    ) -> AppResult<WorkflowRun> {
        self.require_workflow_manage(actor).await?;

        let workflow = self
            .repository
            .find_published_workflow(actor.tenant_id(), workflow_logical_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow '{}' does not have a published version for tenant '{}'",
                    workflow_logical_name,
                    actor.tenant_id()
                ))
            })?;

        if !workflow.is_enabled() {
            return Err(AppError::Conflict(format!(
                "workflow '{}' is disabled",
                workflow.logical_name().as_str()
            )));
        }

        if let Some(payload_object) = trigger_payload.as_object_mut() {
            payload_object
                .entry("triggered_by".to_owned())
                .or_insert_with(|| Value::String(actor.subject().to_owned()));
        }

        let workflow_actor = UserIdentity::new(
            "workflow-runtime",
            "workflow-runtime",
            None,
            actor.tenant_id(),
        );

        match self.execution_mode {
            WorkflowExecutionMode::Inline => {
                self.execute_workflow_definition(&workflow_actor, &workflow, trigger_payload)
                    .await
            }
            WorkflowExecutionMode::Queued => {
                self.enqueue_workflow_definition(&workflow_actor, &workflow, trigger_payload)
                    .await
            }
        }
    }

    /// Dispatches runtime record created trigger across enabled workflows.
    pub async fn dispatch_runtime_record_created(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        record_data: &Value,
    ) -> AppResult<usize> {
        let mut payload = serde_json::json!({
            "entity_logical_name": entity_logical_name,
            "record_id": record_id,
            "id": record_id,
            "record": record_data,
            "data": record_data,
            "event": "created",
        });

        if let Some(payload_object) = payload.as_object_mut()
            && let Some(record_object) = record_data.as_object()
        {
            for (key, value) in record_object {
                payload_object
                    .entry(key.clone())
                    .or_insert_with(|| value.clone());
            }
        }

        self.dispatch_trigger(
            actor,
            WorkflowTrigger::RuntimeRecordCreated {
                entity_logical_name: entity_logical_name.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Dispatches runtime record updated trigger across enabled workflows.
    pub async fn dispatch_runtime_record_updated(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        previous_data: Option<&Value>,
        current_data: &Value,
    ) -> AppResult<usize> {
        let payload = serde_json::json!({
            "entity_logical_name": entity_logical_name,
            "record_id": record_id,
            "id": record_id,
            "event": "updated",
            "previous": previous_data,
            "record": current_data,
            "data": current_data,
        });

        self.dispatch_trigger(
            actor,
            WorkflowTrigger::RuntimeRecordUpdated {
                entity_logical_name: entity_logical_name.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Dispatches runtime record deleted trigger across enabled workflows.
    pub async fn dispatch_runtime_record_deleted(
        &self,
        actor: &UserIdentity,
        entity_logical_name: &str,
        record_id: &str,
        deleted_data: Option<&Value>,
    ) -> AppResult<usize> {
        let payload = serde_json::json!({
            "entity_logical_name": entity_logical_name,
            "record_id": record_id,
            "id": record_id,
            "event": "deleted",
            "record": deleted_data,
            "data": deleted_data,
        });

        self.dispatch_trigger(
            actor,
            WorkflowTrigger::RuntimeRecordDeleted {
                entity_logical_name: entity_logical_name.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Dispatches schedule tick trigger across enabled workflows.
    pub async fn dispatch_schedule_tick(
        &self,
        actor: &UserIdentity,
        schedule_key: &str,
        payload: Option<Value>,
    ) -> AppResult<usize> {
        let event_payload = Self::normalize_schedule_tick_payload(schedule_key, payload)?;

        self.dispatch_trigger(
            actor,
            WorkflowTrigger::ScheduleTick {
                schedule_key: schedule_key.to_owned(),
            },
            event_payload,
        )
        .await
    }

    /// Dispatches inbound webhook trigger across enabled workflows.
    pub async fn dispatch_webhook_received(
        &self,
        tenant_id: TenantId,
        webhook_key: &str,
        mut payload: Value,
    ) -> AppResult<usize> {
        if webhook_key.trim().is_empty() {
            return Err(AppError::Validation(
                "webhook_received trigger requires a non-empty webhook_key".to_owned(),
            ));
        }

        if let Some(payload_object) = payload.as_object_mut() {
            payload_object
                .entry("event".to_owned())
                .or_insert_with(|| Value::String("webhook_received".to_owned()));
            payload_object
                .entry("webhook_key".to_owned())
                .or_insert_with(|| Value::String(webhook_key.to_owned()));
        }

        let webhook_actor =
            UserIdentity::new("workflow-webhook", "workflow-webhook", None, tenant_id);

        self.dispatch_trigger(
            &webhook_actor,
            WorkflowTrigger::WebhookReceived {
                webhook_key: webhook_key.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Dispatches inbound form submission trigger across enabled workflows.
    pub async fn dispatch_form_submitted(
        &self,
        tenant_id: TenantId,
        form_key: &str,
        mut payload: Value,
    ) -> AppResult<usize> {
        if form_key.trim().is_empty() {
            return Err(AppError::Validation(
                "form_submitted trigger requires a non-empty form_key".to_owned(),
            ));
        }

        if let Some(payload_object) = payload.as_object_mut() {
            payload_object
                .entry("event".to_owned())
                .or_insert_with(|| Value::String("form_submitted".to_owned()));
            payload_object
                .entry("form_key".to_owned())
                .or_insert_with(|| Value::String(form_key.to_owned()));
        }

        let form_actor = UserIdentity::new("workflow-form", "workflow-form", None, tenant_id);

        self.dispatch_trigger(
            &form_actor,
            WorkflowTrigger::FormSubmitted {
                form_key: form_key.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Dispatches inbound email trigger across enabled workflows.
    pub async fn dispatch_inbound_email_received(
        &self,
        tenant_id: TenantId,
        mailbox_key: &str,
        mut payload: Value,
    ) -> AppResult<usize> {
        if mailbox_key.trim().is_empty() {
            return Err(AppError::Validation(
                "inbound_email_received trigger requires a non-empty mailbox_key".to_owned(),
            ));
        }

        if let Some(payload_object) = payload.as_object_mut() {
            payload_object
                .entry("event".to_owned())
                .or_insert_with(|| Value::String("inbound_email_received".to_owned()));
            payload_object
                .entry("mailbox_key".to_owned())
                .or_insert_with(|| Value::String(mailbox_key.to_owned()));
        }

        let email_actor = UserIdentity::new("workflow-email", "workflow-email", None, tenant_id);

        self.dispatch_trigger(
            &email_actor,
            WorkflowTrigger::InboundEmailReceived {
                mailbox_key: mailbox_key.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Dispatches approval event trigger across enabled workflows.
    pub async fn dispatch_approval_event_received(
        &self,
        tenant_id: TenantId,
        approval_key: &str,
        mut payload: Value,
    ) -> AppResult<usize> {
        if approval_key.trim().is_empty() {
            return Err(AppError::Validation(
                "approval_event_received trigger requires a non-empty approval_key".to_owned(),
            ));
        }

        if let Some(payload_object) = payload.as_object_mut() {
            payload_object
                .entry("event".to_owned())
                .or_insert_with(|| Value::String("approval_event_received".to_owned()));
            payload_object
                .entry("approval_key".to_owned())
                .or_insert_with(|| Value::String(approval_key.to_owned()));
        }

        let approval_actor =
            UserIdentity::new("workflow-approval", "workflow-approval", None, tenant_id);

        self.dispatch_trigger(
            &approval_actor,
            WorkflowTrigger::ApprovalEventReceived {
                approval_key: approval_key.to_owned(),
            },
            payload,
        )
        .await
    }

    /// Claims and dispatches due built-in scheduler ticks for one worker cycle.
    pub async fn dispatch_due_schedule_ticks(
        &self,
        worker_id: &str,
        lease_seconds: u32,
        tenant_filter: Option<TenantId>,
    ) -> AppResult<WorkflowScheduleTickDrainResult> {
        let triggers = self
            .repository
            .list_enabled_schedule_triggers(tenant_filter)
            .await?;
        if triggers.is_empty() {
            return Ok(WorkflowScheduleTickDrainResult::default());
        }

        let now = Utc::now();
        let mut result = WorkflowScheduleTickDrainResult::default();

        for trigger in triggers {
            let Some(slot) = Self::due_schedule_tick_slot(trigger.schedule_key.as_str(), now)?
            else {
                continue;
            };

            let claimed = self
                .repository
                .claim_schedule_tick(
                    trigger.tenant_id,
                    trigger.schedule_key.as_str(),
                    slot.slot_key.as_str(),
                    slot.tick_at_utc,
                    worker_id,
                    lease_seconds,
                )
                .await?;
            let Some(claimed) = claimed else {
                continue;
            };

            result.claimed_ticks += 1;
            let scheduler_actor = UserIdentity::new(
                "workflow-scheduler",
                "workflow-scheduler",
                None,
                claimed.tenant_id,
            );

            match self
                .dispatch_schedule_tick(
                    &scheduler_actor,
                    claimed.schedule_key.as_str(),
                    Some(serde_json::json!({
                        "tick_at": claimed.scheduled_for.to_rfc3339(),
                        "timezone": "UTC",
                    })),
                )
                .await
            {
                Ok(dispatched) => {
                    result.dispatched_workflows += dispatched;
                    self.repository
                        .complete_schedule_tick(
                            claimed.tenant_id,
                            claimed.schedule_key.as_str(),
                            claimed.slot_key.as_str(),
                            claimed.worker_id.as_str(),
                            claimed.lease_token.as_str(),
                        )
                        .await?;
                }
                Err(error) => {
                    result.released_ticks += 1;
                    self.repository
                        .release_schedule_tick(
                            claimed.tenant_id,
                            claimed.schedule_key.as_str(),
                            claimed.slot_key.as_str(),
                            claimed.worker_id.as_str(),
                            claimed.lease_token.as_str(),
                            error.to_string().as_str(),
                        )
                        .await?;
                }
            }
        }

        Ok(result)
    }

    fn normalize_schedule_tick_payload(
        schedule_key: &str,
        payload: Option<Value>,
    ) -> AppResult<Value> {
        let data = payload.unwrap_or_else(|| serde_json::json!({}));
        let data_object = data.as_object().ok_or_else(|| {
            AppError::Validation("schedule_tick payload must be an object".to_owned())
        })?;

        let reference_now = Utc::now();
        let timezone = data_object
            .get("timezone")
            .and_then(Value::as_str)
            .unwrap_or("UTC")
            .trim()
            .to_owned();
        if timezone.is_empty() {
            return Err(AppError::Validation(
                "schedule_tick timezone must not be empty when provided".to_owned(),
            ));
        }

        let tick_at_source = data_object
            .get("tick_at")
            .or_else(|| data_object.get("tick"))
            .and_then(Value::as_str);

        let (tick_at_utc, tick_source) = match tick_at_source {
            Some(timestamp) => (
                chrono::DateTime::parse_from_rfc3339(timestamp)
                    .map_err(|error| {
                        AppError::Validation(format!(
                            "schedule_tick tick_at must be RFC3339 timestamp: {error}"
                        ))
                    })?
                    .with_timezone(&Utc),
                "payload",
            ),
            None => (reference_now, "system_clock"),
        };

        let clock_skew_seconds = (reference_now - tick_at_utc).num_seconds().abs();
        let clock_skew_within_tolerance =
            clock_skew_seconds <= SCHEDULE_CLOCK_SKEW_TOLERANCE_SECONDS;

        Ok(serde_json::json!({
            "schedule_key": schedule_key,
            "event": "schedule_tick",
            "tick_at_utc": tick_at_utc.to_rfc3339(),
            "tick_source": tick_source,
            "timezone": timezone,
            "clock_skew_seconds": clock_skew_seconds,
            "clock_skew_tolerance_seconds": SCHEDULE_CLOCK_SKEW_TOLERANCE_SECONDS,
            "clock_skew_within_tolerance": clock_skew_within_tolerance,
            "data": data,
        }))
    }

    fn due_schedule_tick_slot(
        schedule_key: &str,
        now: chrono::DateTime<Utc>,
    ) -> AppResult<Option<ScheduleTickSlot>> {
        match schedule_key {
            "hourly" => {
                let tick_at_utc = now
                    .with_minute(0)
                    .and_then(|value| value.with_second(0))
                    .and_then(|value| value.with_nanosecond(0))
                    .ok_or_else(|| {
                        AppError::Internal("failed to normalize hourly schedule tick".to_owned())
                    })?;
                Ok(Some(ScheduleTickSlot {
                    slot_key: tick_at_utc.format("hourly:%Y%m%d%H").to_string(),
                    tick_at_utc,
                }))
            }
            "daily" => {
                let tick_at_utc = now
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .map(|value| chrono::DateTime::<Utc>::from_naive_utc_and_offset(value, Utc))
                    .ok_or_else(|| {
                        AppError::Internal("failed to normalize daily schedule tick".to_owned())
                    })?;
                Ok(Some(ScheduleTickSlot {
                    slot_key: tick_at_utc.format("daily:%Y%m%d").to_string(),
                    tick_at_utc,
                }))
            }
            _ => {
                let Some((hour, minute, weekdays_only)) =
                    Self::parse_utc_time_schedule_key(schedule_key)?
                else {
                    return Ok(None);
                };

                let today_tick = now
                    .date_naive()
                    .and_hms_opt(hour, minute, 0)
                    .map(|value| chrono::DateTime::<Utc>::from_naive_utc_and_offset(value, Utc))
                    .ok_or_else(|| {
                        AppError::Internal(format!(
                            "failed to normalize schedule tick for key '{schedule_key}'"
                        ))
                    })?;

                let tick_at_utc = if now >= today_tick {
                    today_tick
                } else {
                    today_tick - chrono::Duration::days(1)
                };

                if weekdays_only
                    && matches!(
                        tick_at_utc.weekday(),
                        chrono::Weekday::Sat | chrono::Weekday::Sun
                    )
                {
                    return Ok(None);
                }

                Ok(Some(ScheduleTickSlot {
                    slot_key: format!("{}:{}", schedule_key, tick_at_utc.format("%Y%m%d%H%M")),
                    tick_at_utc,
                }))
            }
        }
    }

    fn parse_utc_time_schedule_key(schedule_key: &str) -> AppResult<Option<(u32, u32, bool)>> {
        let (raw_time, weekdays_only, prefix) =
            if let Some(raw_time) = schedule_key.strip_prefix("daily_utc_") {
                (raw_time, false, "daily_utc_")
            } else if let Some(raw_time) = schedule_key.strip_prefix("weekday_utc_") {
                (raw_time, true, "weekday_utc_")
            } else {
                return Ok(None);
            };

        if raw_time.len() != 4 || !raw_time.chars().all(|value| value.is_ascii_digit()) {
            return Err(AppError::Validation(format!(
                "schedule_tick key '{schedule_key}' must use {prefix}HHMM format"
            )));
        }

        let hour = raw_time[..2].parse::<u32>().map_err(|error| {
            AppError::Validation(format!(
                "schedule_tick key '{schedule_key}' has invalid hour: {error}"
            ))
        })?;
        let minute = raw_time[2..].parse::<u32>().map_err(|error| {
            AppError::Validation(format!(
                "schedule_tick key '{schedule_key}' has invalid minute: {error}"
            ))
        })?;

        if hour > 23 || minute > 59 {
            return Err(AppError::Validation(format!(
                "schedule_tick key '{schedule_key}' must use valid UTC HHMM values"
            )));
        }

        Ok(Some((hour, minute, weekdays_only)))
    }

    /// Retries one workflow step for an existing run.
    pub async fn retry_run_step(
        &self,
        actor: &UserIdentity,
        workflow_logical_name: &str,
        run_id: &str,
        step_path: &str,
    ) -> AppResult<WorkflowRun> {
        self.require_workflow_manage(actor).await?;

        let run = self
            .repository
            .find_run(actor.tenant_id(), run_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow run '{}' does not exist for tenant '{}'",
                    run_id,
                    actor.tenant_id()
                ))
            })?;

        let workflow = self
            .repository
            .find_published_workflow_version(
                actor.tenant_id(),
                workflow_logical_name,
                run.workflow_version,
            )
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "workflow '{}' published version {} does not exist for tenant '{}'",
                    workflow_logical_name,
                    run.workflow_version,
                    actor.tenant_id()
                ))
            })?;

        if run.workflow_logical_name != workflow.logical_name().as_str() {
            return Err(AppError::Validation(format!(
                "run '{}' does not belong to workflow '{}'",
                run_id, workflow_logical_name
            )));
        }

        let workflow_actor = UserIdentity::new(
            "workflow-runtime",
            "workflow-runtime",
            None,
            actor.tenant_id(),
        );

        self.retry_step_for_run(&workflow_actor, &workflow, &run, step_path)
            .await
    }
}
