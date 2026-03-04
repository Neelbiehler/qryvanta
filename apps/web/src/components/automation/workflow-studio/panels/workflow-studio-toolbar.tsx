import Link from "next/link";

import { Button, StatusBadge, buttonVariants } from "@qryvanta/ui";

type WorkflowStudioToolbarProps = {
  selectedWorkflow: string;
  workspaceMode: "edit" | "history";
  validationErrorCount: number;
  errorMessage: string | null;
  statusMessage: string | null;
  undoDisabled: boolean;
  redoDisabled: boolean;
  showBuilderPanel: boolean;
  onUndo: () => void;
  onRedo: () => void;
  onOpenNodePicker: () => void;
  onToggleBuilderPanel: () => void;
};

export function WorkflowStudioToolbar({
  selectedWorkflow,
  workspaceMode,
  validationErrorCount,
  errorMessage,
  statusMessage,
  undoDisabled,
  redoDisabled,
  showBuilderPanel,
  onUndo,
  onRedo,
  onOpenNodePicker,
  onToggleBuilderPanel,
}: WorkflowStudioToolbarProps) {
  const isHistoryMode = workspaceMode === "history";

  return (
    <div className="absolute inset-x-0 top-0 z-40 flex h-12 items-center gap-2 border-b border-zinc-200 bg-white/95 px-3 backdrop-blur">
      {selectedWorkflow ? (
        <span className="truncate rounded border border-zinc-200 bg-zinc-50 px-2 py-0.5 text-xs font-medium text-zinc-700">
          {selectedWorkflow}
        </span>
      ) : (
        <span className="text-xs text-zinc-400">No flow selected</span>
      )}
      <StatusBadge tone="neutral">
        {isHistoryMode ? "Step history" : "Edit mode"}
      </StatusBadge>
      {validationErrorCount > 0 && (
        <StatusBadge tone="warning">
          {validationErrorCount} error{validationErrorCount !== 1 ? "s" : ""}
        </StatusBadge>
      )}
      {errorMessage ? (
        <p className="max-w-xs truncate rounded border border-red-200 bg-red-50 px-2 py-0.5 text-xs text-red-700">
          {errorMessage}
        </p>
      ) : null}
      {statusMessage ? (
        <p className="max-w-xs truncate rounded border border-emerald-200 bg-emerald-50 px-2 py-0.5 text-xs text-emerald-700">
          {statusMessage}
        </p>
      ) : null}

      <div className="ml-auto flex items-center gap-1.5">
        {!isHistoryMode ? (
          <>
            <Button
              type="button"
              size="sm"
              variant="outline"
              onClick={onUndo}
              disabled={undoDisabled}
            >
              Undo
            </Button>
            <Button
              type="button"
              size="sm"
              variant="outline"
              onClick={onRedo}
              disabled={redoDisabled}
            >
              Redo
            </Button>
            <div className="mx-1 h-5 w-px bg-zinc-200" />
            <Button type="button" size="sm" variant="outline" onClick={onOpenNodePicker}>
              Add step
            </Button>
          </>
        ) : null}
        <Button
          type="button"
          size="sm"
          variant="outline"
          onClick={onToggleBuilderPanel}
        >
          {showBuilderPanel ? "Hide panel" : "Show panel"}
        </Button>
        <div className="mx-1 h-5 w-px bg-zinc-200" />
        {selectedWorkflow && !isHistoryMode ? (
          <Link
            href={`/maker/automation/${encodeURIComponent(selectedWorkflow)}/history`}
            className={buttonVariants({ size: "sm", variant: "outline" })}
          >
            History
          </Link>
        ) : null}
        {selectedWorkflow && isHistoryMode ? (
          <>
            <Link
              href={`/maker/automation/${encodeURIComponent(selectedWorkflow)}/history`}
              className={buttonVariants({ size: "sm", variant: "outline" })}
            >
              History list
            </Link>
            <Link
              href={`/maker/automation/${encodeURIComponent(selectedWorkflow)}/edit`}
              className={buttonVariants({ size: "sm", variant: "outline" })}
            >
              Edit
            </Link>
          </>
        ) : null}
      </div>
    </div>
  );
}
