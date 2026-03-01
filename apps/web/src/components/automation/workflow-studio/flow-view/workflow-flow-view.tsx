"use client";

import { Plus } from "lucide-react";

import { Button } from "@qryvanta/ui";

import {
  type CatalogInsertMode,
  type DraftWorkflowStep,
  type DynamicTokenOption,
  type TriggerType,
} from "@/components/automation/workflow-studio/model";
import {
  FlowConnector,
  type SharedStepProps,
  StepBlock,
  TriggerCard,
} from "@/components/automation/workflow-studio/flow-view/flow-view-components";
import type {
  RetryWorkflowStepStrategyDto,
  WorkflowRunStepTraceResponse,
} from "@/lib/api";

export type WorkflowFlowViewProps = {
  steps: DraftWorkflowStep[];
  triggerType: TriggerType;
  triggerEntityLogicalName: string;
  expandedNodeId: string | null;
  onExpandNode: (nodeId: string | null) => void;
  onUpdateStep: (
    stepId: string,
    updater: (step: DraftWorkflowStep) => DraftWorkflowStep,
  ) => void;
  onRemoveStep: (stepId: string) => void;
  onDuplicateStep: (stepId: string) => void;
  onOpenNodePicker: (mode: CatalogInsertMode, stepId?: string) => void;
  getAvailableTokens: (stepId: string) => DynamicTokenOption[];
  onTriggerTypeChange: (type: TriggerType) => void;
  onTriggerEntityChange: (entity: string) => void;
  runtimeEntityOptions: Array<{ value: string; label: string }>;
  triggerFieldPathSuggestions: string[];
  getEntityFieldPathSuggestions: (entityLogicalName: string) => string[];
  stepTraceByPath: Record<string, WorkflowRunStepTraceResponse>;
  stepPathByStepId: Record<string, string>;
  isRetryingStep: boolean;
  onRetryStep: (
    stepPath: string,
    strategy: RetryWorkflowStepStrategyDto,
    backoffMs?: number,
  ) => void;
};

export function WorkflowFlowView({
  steps,
  triggerType,
  triggerEntityLogicalName,
  expandedNodeId,
  onExpandNode,
  onUpdateStep,
  onRemoveStep,
  onDuplicateStep,
  onOpenNodePicker,
  getAvailableTokens,
  onTriggerTypeChange,
  onTriggerEntityChange,
  runtimeEntityOptions,
  triggerFieldPathSuggestions,
  getEntityFieldPathSuggestions,
  stepTraceByPath,
  stepPathByStepId,
  isRetryingStep,
  onRetryStep,
}: WorkflowFlowViewProps) {
  const shared: SharedStepProps = {
    expandedNodeId,
    onExpandNode,
    onUpdateStep,
    onRemoveStep,
    onDuplicateStep,
    onOpenNodePicker,
    getAvailableTokens,
    runtimeEntityOptions,
    triggerFieldPathSuggestions,
    getEntityFieldPathSuggestions,
    stepTraceByPath,
    stepPathByStepId,
    isRetryingStep,
    onRetryStep,
  };

  return (
    <div className="h-full overflow-y-auto bg-slate-50/60">
      <div className="flex flex-col items-center px-6 py-10">
        <div className="w-full max-w-2xl">
          <TriggerCard
            triggerType={triggerType}
            triggerEntityLogicalName={triggerEntityLogicalName}
            isExpanded={expandedNodeId === "trigger"}
            onToggle={() => onExpandNode(expandedNodeId === "trigger" ? null : "trigger")}
            onTriggerTypeChange={onTriggerTypeChange}
            onTriggerEntityChange={onTriggerEntityChange}
            runtimeEntityOptions={runtimeEntityOptions}
          />

          <FlowConnector onAdd={() => onOpenNodePicker("root")} />

          {steps.length > 0 ? (
            steps.map((step) => <StepBlock key={step.id} step={step} {...shared} />)
          ) : (
            <div className="flex flex-col items-center gap-3 rounded-xl border border-dashed border-zinc-300 bg-white py-10 text-center">
              <div className="flex size-10 items-center justify-center rounded-full bg-zinc-100 text-zinc-400">
                <Plus className="size-5" />
              </div>
              <div>
                <p className="text-sm font-medium text-zinc-600">No steps yet</p>
                <p className="mt-0.5 text-xs text-zinc-400">
                  Press <kbd className="rounded border border-zinc-200 bg-zinc-100 px-1 py-0.5 font-mono text-[10px]">A</kbd> or click + to add your first step
                </p>
              </div>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => onOpenNodePicker("root")}
              >
                <Plus className="mr-1.5 size-3.5" />
                Add step
              </Button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
