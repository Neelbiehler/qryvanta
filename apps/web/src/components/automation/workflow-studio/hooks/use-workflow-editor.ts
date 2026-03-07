import { useState } from "react";

import {
  apiFetch,
  type SaveWorkflowRequest,
  type WorkflowConditionOperatorDto,
  type WorkflowResponse,
  type WorkflowStepDto,
} from "@/lib/api";
import {
  parseDraftArrayItems,
  parseDraftObjectFields,
  parseDraftValue,
  parseJsonStringMap,
  parseJsonValue,
  type DraftWorkflowStep,
  type TriggerType,
} from "@/components/automation/workflow-studio/model";

type UseWorkflowEditorInput = {
  onResetMessages: () => void;
  onStatusMessage: (message: string | null) => void;
  onErrorMessage: (message: string | null) => void;
  onRefresh: () => void;
};

export function useWorkflowEditor({
  onResetMessages,
  onStatusMessage,
  onErrorMessage,
  onRefresh,
}: UseWorkflowEditorInput) {
  const [logicalName, setLogicalName] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [description, setDescription] = useState("");
  const [triggerType, setTriggerType] = useState<TriggerType>("manual");
  const [triggerEntityLogicalName, setTriggerEntityLogicalName] = useState("");
  const [maxAttempts, setMaxAttempts] = useState("3");
  const [workflowLifecycleState, setWorkflowLifecycleState] = useState<
    WorkflowResponse["lifecycle_state"]
  >("draft");
  const [publishedVersion, setPublishedVersion] = useState<number | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [isPublishing, setIsPublishing] = useState(false);
  const [isDisabling, setIsDisabling] = useState(false);

  function compileStep(step: DraftWorkflowStep): WorkflowStepDto {
    if (step.type === "log_message") {
      if (step.message.trim().length === 0) {
        throw new Error("Log message step requires a non-empty message.");
      }

      return {
        type: "log_message",
        message: step.message,
      };
    }

    if (step.type === "create_runtime_record") {
      if (step.entityLogicalName.trim().length === 0) {
        throw new Error("Create record step requires an entity logical name.");
      }

      return {
        type: "create_runtime_record",
        entity_logical_name: step.entityLogicalName,
        data: parseDraftObjectFields(step.dataFields, "Create record step data"),
      };
    }

    if (step.type === "update_runtime_record") {
      if (step.entityLogicalName.trim().length === 0) {
        throw new Error("Update record step requires an entity logical name.");
      }
      if (step.recordId.trim().length === 0) {
        throw new Error("Update record step requires a record id.");
      }

      return {
        type: "update_runtime_record",
        entity_logical_name: step.entityLogicalName,
        record_id: step.recordId,
        data: parseDraftObjectFields(step.dataFields, "Update record step data"),
      };
    }

    if (step.type === "delete_runtime_record") {
      if (step.entityLogicalName.trim().length === 0) {
        throw new Error("Delete record step requires an entity logical name.");
      }
      if (step.recordId.trim().length === 0) {
        throw new Error("Delete record step requires a record id.");
      }

      return {
        type: "delete_runtime_record",
        entity_logical_name: step.entityLogicalName,
        record_id: step.recordId,
      };
    }

    if (step.type === "send_email") {
      if (step.to.trim().length === 0) {
        throw new Error("Send email step requires a recipient address.");
      }

      if (step.subject.trim().length === 0) {
        throw new Error("Send email step requires a subject.");
      }

      if (step.body.trim().length === 0) {
        throw new Error("Send email step requires a body.");
      }

      return {
        type: "send_email",
        to: step.to,
        subject: step.subject,
        body: step.body,
        html_body: step.htmlBody.trim().length > 0 ? step.htmlBody : null,
      };
    }

    if (step.type === "http_request") {
      if (step.method.trim().length === 0) {
        throw new Error("HTTP request step requires a method.");
      }

      if (step.url.trim().length === 0) {
        throw new Error("HTTP request step requires a URL.");
      }

      return {
        type: "http_request",
        method: step.method,
        url: step.url,
        headers: parseJsonStringMap(step.headersJson, "HTTP request headers"),
        header_secret_refs: parseJsonStringMap(
          step.headerSecretRefsJson,
          "HTTP request secret headers",
        ),
        body:
          step.bodyMode === "none"
            ? null
            : step.bodyMode === "object"
              ? parseDraftObjectFields(step.bodyFields, "HTTP request body")
              : step.bodyMode === "array"
                ? parseDraftArrayItems(step.bodyArrayItems, "HTTP request body")
                : step.bodyMode === "scalar"
                  ? (parseDraftValue(
                      step.bodyScalarKind,
                      step.bodyScalarValue,
                      "HTTP request body",
                    ) as unknown)
              : (parseJsonValue(step.bodyJson, "HTTP request body") as unknown),
      };
    }

    if (step.type === "webhook") {
      if (step.endpoint.trim().length === 0) {
        throw new Error("Webhook step requires an endpoint.");
      }

      if (step.event.trim().length === 0) {
        throw new Error("Webhook step requires an event name.");
      }

      return {
        type: "webhook",
        endpoint: step.endpoint,
        event: step.event,
        headers: parseJsonStringMap(step.headersJson, "Webhook headers"),
        header_secret_refs: parseJsonStringMap(
          step.headerSecretRefsJson,
          "Webhook secret headers",
        ),
        payload: parseDraftObjectFields(step.payloadFields, "Webhook payload"),
      };
    }

    if (step.type === "assign_owner") {
      if (step.entityLogicalName.trim().length === 0) {
        throw new Error("Assign owner step requires an entity logical name.");
      }
      if (step.recordId.trim().length === 0) {
        throw new Error("Assign owner step requires a record id.");
      }
      if (step.ownerId.trim().length === 0) {
        throw new Error("Assign owner step requires an owner id.");
      }

      return {
        type: "assign_owner",
        entity_logical_name: step.entityLogicalName,
        record_id: step.recordId,
        owner_id: step.ownerId,
        reason: step.reason.trim().length > 0 ? step.reason : null,
      };
    }

    if (step.type === "approval_request") {
      if (step.entityLogicalName.trim().length === 0) {
        throw new Error("Approval request step requires an entity logical name.");
      }
      if (step.recordId.trim().length === 0) {
        throw new Error("Approval request step requires a record id.");
      }
      if (step.requestType.trim().length === 0) {
        throw new Error("Approval request step requires a request type.");
      }

      return {
        type: "approval_request",
        entity_logical_name: step.entityLogicalName,
        record_id: step.recordId,
        request_type: step.requestType,
        requested_by: step.requestedBy.trim().length > 0 ? step.requestedBy : null,
        approver_id: step.approverId.trim().length > 0 ? step.approverId : null,
        reason: step.reason.trim().length > 0 ? step.reason : null,
        payload: parseDraftObjectFields(step.payloadFields, "Approval request payload"),
      };
    }

    if (step.type === "delay") {
      const durationMs = Number.parseInt(step.durationMs, 10);
      if (!Number.isFinite(durationMs) || durationMs <= 0) {
        throw new Error("Delay step requires a positive duration in milliseconds.");
      }

      return {
        type: "delay",
        duration_ms: durationMs,
        reason: step.reason.trim().length > 0 ? step.reason : null,
      };
    }

    if (step.fieldPath.trim().length === 0) {
      throw new Error("Condition step requires a payload field path.");
    }

    const thenSteps = step.thenSteps.map(compileStep);
    const elseSteps = step.elseSteps.map(compileStep);
    if (thenSteps.length === 0 && elseSteps.length === 0) {
      throw new Error("Condition step requires at least one branch step.");
    }

    const value =
      step.operator === "exists"
        ? null
        : (parseDraftValue(step.valueKind, step.valueText, "Condition value") as unknown);

    return {
      type: "condition",
      field_path: step.fieldPath,
      operator: step.operator as WorkflowConditionOperatorDto,
      value,
      then_label: step.thenLabel.trim().length > 0 ? step.thenLabel.trim() : null,
      else_label: step.elseLabel.trim().length > 0 ? step.elseLabel.trim() : null,
      then_steps: thenSteps,
      else_steps: elseSteps,
    };
  }

  function buildSavePayload(steps: DraftWorkflowStep[]): SaveWorkflowRequest {
    const parsedMaxAttempts = Number.parseInt(maxAttempts, 10);
    if (!Number.isFinite(parsedMaxAttempts)) {
      throw new Error("Max attempts must be a number.");
    }

    const compiledSteps = steps.map(compileStep);
    if (compiledSteps.length === 0) {
      throw new Error("Flow canvas requires at least one step.");
    }

    return {
      logical_name: logicalName,
      display_name: displayName,
      description: description.trim().length > 0 ? description : null,
      trigger_type: triggerType,
      trigger_entity_logical_name:
        triggerType !== "manual" &&
        triggerEntityLogicalName.trim().length > 0
          ? triggerEntityLogicalName
          : null,
      steps: compiledSteps,
      max_attempts: parsedMaxAttempts,
    };
  }

  function applyWorkflowResponse(workflow: WorkflowResponse) {
    setWorkflowLifecycleState(workflow.lifecycle_state);
    setPublishedVersion(workflow.published_version ?? null);
  }

  async function saveWorkflow(steps: DraftWorkflowStep[]) {
    onResetMessages();
    setIsSaving(true);

    try {
      const response = await apiFetch("/api/workflows", {
        method: "POST",
        body: JSON.stringify(buildSavePayload(steps)),
      });

      if (!response.ok) {
        const responsePayload = (await response.json()) as { message?: string };
        onErrorMessage(responsePayload.message ?? "Unable to save workflow draft.");
        return;
      }

      const workflow = (await response.json()) as WorkflowResponse;
      applyWorkflowResponse(workflow);
      onStatusMessage("Workflow draft saved.");
      onRefresh();
    } catch (error) {
      onErrorMessage(
        error instanceof Error ? error.message : "Unable to save workflow draft.",
      );
    } finally {
      setIsSaving(false);
    }
  }

  async function publishWorkflow(steps: DraftWorkflowStep[]) {
    onResetMessages();
    setIsPublishing(true);

    try {
      const draftResponse = await apiFetch("/api/workflows", {
        method: "POST",
        body: JSON.stringify(buildSavePayload(steps)),
      });

      if (!draftResponse.ok) {
        const responsePayload = (await draftResponse.json()) as { message?: string };
        onErrorMessage(responsePayload.message ?? "Unable to save workflow draft.");
        return;
      }

      const draftWorkflow = (await draftResponse.json()) as WorkflowResponse;
      const publishResponse = await apiFetch(
        `/api/workflows/${encodeURIComponent(draftWorkflow.logical_name)}/publish`,
        { method: "POST" },
      );

      if (!publishResponse.ok) {
        const responsePayload = (await publishResponse.json()) as { message?: string };
        onErrorMessage(responsePayload.message ?? "Unable to publish workflow.");
        return;
      }

      const workflow = (await publishResponse.json()) as WorkflowResponse;
      applyWorkflowResponse(workflow);
      onStatusMessage("Workflow published.");
      onRefresh();
    } catch (error) {
      onErrorMessage(
        error instanceof Error ? error.message : "Unable to publish workflow.",
      );
    } finally {
      setIsPublishing(false);
    }
  }

  async function disableWorkflow() {
    onResetMessages();
    setIsDisabling(true);

    try {
      if (logicalName.trim().length === 0) {
        throw new Error("Save the workflow before disabling it.");
      }

      const response = await apiFetch(
        `/api/workflows/${encodeURIComponent(logicalName)}/disable`,
        { method: "POST" },
      );

      if (!response.ok) {
        const responsePayload = (await response.json()) as { message?: string };
        onErrorMessage(responsePayload.message ?? "Unable to disable workflow.");
        return;
      }

      const workflow = (await response.json()) as WorkflowResponse;
      applyWorkflowResponse(workflow);
      onStatusMessage("Workflow disabled.");
      onRefresh();
    } catch (error) {
      onErrorMessage(
        error instanceof Error ? error.message : "Unable to disable workflow.",
      );
    } finally {
      setIsDisabling(false);
    }
  }

  return {
    logicalName,
    setLogicalName,
    displayName,
    setDisplayName,
    description,
    setDescription,
    triggerType,
    setTriggerType,
    triggerEntityLogicalName,
    setTriggerEntityLogicalName,
    maxAttempts,
    setMaxAttempts,
    workflowLifecycleState,
    setWorkflowLifecycleState,
    publishedVersion,
    setPublishedVersion,
    isSaving,
    isPublishing,
    isDisabling,
    saveWorkflow,
    publishWorkflow,
    disableWorkflow,
  };
}
