import { useState } from "react";

import {
  apiFetch,
  type SaveWorkflowRequest,
  type WorkflowConditionOperatorDto,
  type WorkflowStepDto,
} from "@/lib/api";
import {
  firstActionFromSteps,
  parseJsonObject,
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
  const [isEnabled, setIsEnabled] = useState(true);
  const [isSaving, setIsSaving] = useState(false);

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
        data: parseJsonObject(step.dataJson, "Create record step data"),
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
        : (parseJsonValue(step.valueJson, "Condition value") as unknown);

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

  async function saveWorkflow(steps: DraftWorkflowStep[]) {
    onResetMessages();
    setIsSaving(true);

    try {
      const parsedMaxAttempts = Number.parseInt(maxAttempts, 10);
      if (!Number.isFinite(parsedMaxAttempts)) {
        throw new Error("Max attempts must be a number.");
      }

      const compiledSteps = steps.map(compileStep);
      if (compiledSteps.length === 0) {
        throw new Error("Flow canvas requires at least one step.");
      }

      const firstAction = firstActionFromSteps(compiledSteps);
      if (!firstAction) {
        throw new Error("Flow canvas must contain at least one executable action step.");
      }

      const payload: SaveWorkflowRequest = {
        logical_name: logicalName,
        display_name: displayName,
        description: description.trim().length > 0 ? description : null,
        trigger_type: triggerType,
        trigger_entity_logical_name:
          triggerType === "runtime_record_created" &&
          triggerEntityLogicalName.trim().length > 0
            ? triggerEntityLogicalName
            : null,
        action_type: firstAction.actionType,
        action_entity_logical_name: firstAction.actionEntityLogicalName,
        action_payload: firstAction.actionPayload,
        steps: compiledSteps,
        max_attempts: parsedMaxAttempts,
        is_enabled: isEnabled,
      };

      const response = await apiFetch("/api/workflows", {
        method: "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const responsePayload = (await response.json()) as { message?: string };
        onErrorMessage(responsePayload.message ?? "Unable to save workflow.");
        return;
      }

      onStatusMessage("Workflow saved.");
      onRefresh();
    } catch (error) {
      onErrorMessage(
        error instanceof Error ? error.message : "Unable to save workflow.",
      );
    } finally {
      setIsSaving(false);
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
    isEnabled,
    setIsEnabled,
    isSaving,
    saveWorkflow,
  };
}
