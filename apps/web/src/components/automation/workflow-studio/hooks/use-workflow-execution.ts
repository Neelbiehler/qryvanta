import { type FormEvent, useState } from "react";

import {
  apiFetch,
  type ExecuteWorkflowRequest,
  type WorkflowRunAttemptResponse,
} from "@/lib/api";
import { parseJsonObject } from "@/components/automation/workflow-studio/model";

type UseWorkflowExecutionInput = {
  selectedWorkflow: string;
  onResetMessages: () => void;
  onStatusMessage: (message: string | null) => void;
  onErrorMessage: (message: string | null) => void;
  onRefresh: () => void;
};

export function useWorkflowExecution({
  selectedWorkflow,
  onResetMessages,
  onStatusMessage,
  onErrorMessage,
  onRefresh,
}: UseWorkflowExecutionInput) {
  const [executePayload, setExecutePayload] = useState(
    JSON.stringify({ manual: true }, null, 2),
  );
  const [attemptsByRun, setAttemptsByRun] = useState<
    Record<string, WorkflowRunAttemptResponse[]>
  >({});
  const [expandedRunId, setExpandedRunId] = useState<string | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);

  async function handleExecuteWorkflow(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedWorkflow) {
      onErrorMessage("Select a workflow first.");
      return;
    }

    onResetMessages();
    setIsExecuting(true);
    try {
      const triggerPayload = parseJsonObject(executePayload, "Trigger payload");
      const payload: ExecuteWorkflowRequest = {
        trigger_payload: triggerPayload,
      };

      const response = await apiFetch(`/api/workflows/${selectedWorkflow}/execute`, {
        method: "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const responsePayload = (await response.json()) as { message?: string };
        onErrorMessage(responsePayload.message ?? "Unable to execute workflow.");
        return;
      }

      onStatusMessage("Workflow executed.");
      onRefresh();
    } catch (error) {
      onErrorMessage(
        error instanceof Error ? error.message : "Unable to execute workflow.",
      );
    } finally {
      setIsExecuting(false);
    }
  }

  async function toggleAttempts(runId: string) {
    if (expandedRunId === runId) {
      setExpandedRunId(null);
      return;
    }

    if (!attemptsByRun[runId]) {
      const response = await apiFetch(`/api/workflows/runs/${runId}/attempts`);
      if (!response.ok) {
        onErrorMessage("Unable to load workflow run attempts.");
        return;
      }

      const attempts = (await response.json()) as WorkflowRunAttemptResponse[];
      setAttemptsByRun((current) => ({ ...current, [runId]: attempts }));
    }

    setExpandedRunId(runId);
  }

  return {
    executePayload,
    setExecutePayload,
    attemptsByRun,
    expandedRunId,
    isExecuting,
    handleExecuteWorkflow,
    toggleAttempts,
  };
}
