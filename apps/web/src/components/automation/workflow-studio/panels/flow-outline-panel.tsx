import { Button } from "@qryvanta/ui";

import {
  summarizeStep,
  type DraftWorkflowStep,
} from "@/components/automation/workflow-studio/model";

type FlowOutlinePanelProps = {
  steps: DraftWorkflowStep[];
  selectedStepId: string | null;
  onSelectStep: (stepId: string) => void;
  onAddBranchStep: (
    conditionStepId: string,
    branch: "then" | "else",
    stepType: DraftWorkflowStep["type"],
  ) => void;
};

export function FlowOutlinePanel({
  steps,
  selectedStepId,
  onSelectStep,
  onAddBranchStep,
}: FlowOutlinePanelProps) {
  function renderStep(
    step: DraftWorkflowStep,
    depth: number,
    branchLabel?: string,
  ) {
    const isSelected = selectedStepId === step.id;

    if (step.type !== "condition") {
      return (
        <div key={step.id} className="space-y-2">
          {branchLabel ? (
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-500">
              {branchLabel}
            </p>
          ) : null}
          <button
            type="button"
            className={`w-full rounded-lg border p-3 text-left transition ${
              isSelected
                ? "border-emerald-500 bg-emerald-50"
                : "border-zinc-200 bg-white hover:border-emerald-300"
            }`}
            style={{ marginLeft: `${depth * 14}px` }}
            onClick={() => onSelectStep(step.id)}
          >
            <p className="text-xs font-semibold uppercase tracking-wide text-emerald-700">
              {step.type.replaceAll("_", " ")}
            </p>
            <p className="mt-1 text-sm text-zinc-900">{summarizeStep(step)}</p>
          </button>
        </div>
      );
    }

    return (
      <div
        key={step.id}
        className="space-y-2"
        style={{ marginLeft: `${depth * 14}px` }}
      >
        <button
          type="button"
          className={`w-full rounded-lg border p-3 text-left transition ${
            isSelected
              ? "border-emerald-500 bg-emerald-50"
              : "border-zinc-200 bg-white hover:border-emerald-300"
          }`}
          onClick={() => onSelectStep(step.id)}
        >
          <p className="text-xs font-semibold uppercase tracking-wide text-emerald-700">
            Condition
          </p>
          <p className="mt-1 text-sm text-zinc-900">{summarizeStep(step)}</p>
        </button>

        <div className="grid gap-2 md:grid-cols-2">
          <div className="space-y-2 rounded-md border border-emerald-100 bg-white p-2">
            <p className="text-[11px] font-semibold uppercase tracking-wide text-emerald-700">
              If {step.thenLabel || "Yes"}
            </p>
            {step.thenSteps.map((childStep) => renderStep(childStep, depth + 1))}
            <div className="flex flex-wrap gap-1">
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => onAddBranchStep(step.id, "then", "log_message")}
              >
                + Log
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() =>
                  onAddBranchStep(step.id, "then", "create_runtime_record")
                }
              >
                + Create
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => onAddBranchStep(step.id, "then", "condition")}
              >
                + Condition
              </Button>
            </div>
          </div>

          <div className="space-y-2 rounded-md border border-zinc-200 bg-white p-2">
            <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
              If {step.elseLabel || "No"}
            </p>
            {step.elseSteps.map((childStep) => renderStep(childStep, depth + 1))}
            <div className="flex flex-wrap gap-1">
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => onAddBranchStep(step.id, "else", "log_message")}
              >
                + Log
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() =>
                  onAddBranchStep(step.id, "else", "create_runtime_record")
                }
              >
                + Create
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => onAddBranchStep(step.id, "else", "condition")}
              >
                + Condition
              </Button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
        Flow Outline
      </p>
      <div className="space-y-2">{steps.map((step) => renderStep(step, 0))}</div>
    </div>
  );
}
