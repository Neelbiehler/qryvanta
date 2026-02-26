import { Button, StatusBadge } from "@qryvanta/ui";

import { formatUtcDateTime } from "@/lib/date-format";
import type {
  WorkflowRunAttemptResponse,
  WorkflowRunResponse,
} from "@/lib/api";

type RunHistoryPanelProps = {
  selectedWorkflow: string;
  selectedWorkflowRuns: WorkflowRunResponse[];
  expandedRunId: string | null;
  attemptsByRun: Record<string, WorkflowRunAttemptResponse[]>;
  onToggleAttempts: (runId: string) => void;
};

export function RunHistoryPanel({
  selectedWorkflow,
  selectedWorkflowRuns,
  expandedRunId,
  attemptsByRun,
  onToggleAttempts,
}: RunHistoryPanelProps) {
  return (
    <div className="space-y-2">
      <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
        Workflow History
      </p>
      {selectedWorkflowRuns.length > 0 ? (
        <div className="max-h-[520px] space-y-2 overflow-y-auto pr-1">
          {selectedWorkflowRuns.map((run) => (
            <div key={run.run_id} className="rounded-md border border-zinc-200 p-2 text-xs">
              <div className="flex flex-wrap items-center gap-2">
                <span className="font-mono text-[11px]">{run.run_id}</span>
                <StatusBadge tone="neutral">{run.status}</StatusBadge>
                <span>{formatUtcDateTime(run.started_at)}</span>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={() => onToggleAttempts(run.run_id)}
                >
                  {expandedRunId === run.run_id ? "Hide" : "Attempts"}
                </Button>
              </div>
              {expandedRunId === run.run_id ? (
                <div className="mt-1 space-y-1 text-[11px] text-zinc-600">
                  {(attemptsByRun[run.run_id] ?? []).map((attempt) => (
                    <p key={`${attempt.run_id}-${attempt.attempt_number}`}>
                      #{attempt.attempt_number} {attempt.status}
                      {attempt.error_message ? ` - ${attempt.error_message}` : ""}
                    </p>
                  ))}
                </div>
              ) : null}
            </div>
          ))}
        </div>
      ) : (
        <p className="text-xs text-zinc-500">
          {selectedWorkflow
            ? "No runs for this workflow yet."
            : "Select a workflow to view run history."}
        </p>
      )}
    </div>
  );
}
