import type { FormEvent } from "react";

import { Button, Input, Select, Separator, Textarea } from "@qryvanta/ui";

import { RunHistoryPanel } from "@/components/automation/workflow-studio/panels/run-history-panel";
import { FlowOutlinePanel } from "@/components/automation/workflow-studio/panels/flow-outline-panel";
import type {
  WorkflowRunAttemptResponse,
  WorkflowRunResponse,
  WorkflowResponse,
} from "@/lib/api";
import {
  STEP_LIBRARY,
  type CatalogInsertMode,
  type DraftWorkflowStep,
  type FlowTemplateCategory,
  type FlowTemplateId,
} from "@/components/automation/workflow-studio/model";

type TemplateOption = {
  id: FlowTemplateId;
  label: string;
  description: string;
  category: FlowTemplateCategory;
};

type WorkflowBuilderPanelProps = {
  open: boolean;
  workflowQuery: string;
  onWorkflowQueryChange: (value: string) => void;
  filteredWorkflows: WorkflowResponse[];
  selectedWorkflow: string;
  onLoadWorkflow: (workflow: WorkflowResponse) => void;
  onOpenWorkflowHistory: (workflow: WorkflowResponse) => void;
  onResetBuilder: () => void;
  workflowWorkspaceMode: "edit" | "history";
  onSaveWorkflow: (event: FormEvent<HTMLFormElement>) => void;
  logicalName: string;
  onLogicalNameChange: (value: string) => void;
  displayName: string;
  onDisplayNameChange: (value: string) => void;
  description: string;
  onDescriptionChange: (value: string) => void;
  maxAttempts: string;
  onMaxAttemptsChange: (value: string) => void;
  isEnabled: boolean;
  onEnabledChange: (value: boolean) => void;
  catalogQuery: string;
  onCatalogQueryChange: (value: string) => void;
  catalogCategory: "all" | FlowTemplateCategory;
  onCatalogCategoryChange: (value: "all" | FlowTemplateCategory) => void;
  catalogInsertMode: CatalogInsertMode;
  onCatalogInsertModeChange: (value: CatalogInsertMode) => void;
  canInsertIntoConditionBranch: boolean;
  filteredTemplates: TemplateOption[];
  onInsertTemplate: (templateId: FlowTemplateId) => void;
  onAddRootStep: (stepType: DraftWorkflowStep["type"]) => void;
  isSaving: boolean;
  onExecuteWorkflow: (event: FormEvent<HTMLFormElement>) => void;
  onExecutionWorkflowChange: (workflowLogicalName: string) => void;
  workflows: WorkflowResponse[];
  executePayload: string;
  onExecutePayloadChange: (value: string) => void;
  isExecuting: boolean;
  steps: DraftWorkflowStep[];
  selectedStepId: string | null;
  onSelectStep: (stepId: string) => void;
  onAddBranchStep: (
    conditionStepId: string,
    branch: "then" | "else",
    stepType: DraftWorkflowStep["type"],
  ) => void;
  selectedWorkflowRuns: WorkflowRunResponse[];
  expandedRunId: string | null;
  attemptsByRun: Record<string, WorkflowRunAttemptResponse[]>;
  onToggleAttempts: (runId: string) => void;
};

export function WorkflowBuilderPanel({
  open,
  workflowQuery,
  onWorkflowQueryChange,
  filteredWorkflows,
  selectedWorkflow,
  onLoadWorkflow,
  onOpenWorkflowHistory,
  onResetBuilder,
  workflowWorkspaceMode,
  onSaveWorkflow,
  logicalName,
  onLogicalNameChange,
  displayName,
  onDisplayNameChange,
  description,
  onDescriptionChange,
  maxAttempts,
  onMaxAttemptsChange,
  isEnabled,
  onEnabledChange,
  catalogQuery,
  onCatalogQueryChange,
  catalogCategory,
  onCatalogCategoryChange,
  catalogInsertMode,
  onCatalogInsertModeChange,
  canInsertIntoConditionBranch,
  filteredTemplates,
  onInsertTemplate,
  onAddRootStep,
  isSaving,
  onExecuteWorkflow,
  onExecutionWorkflowChange,
  workflows,
  executePayload,
  onExecutePayloadChange,
  isExecuting,
  steps,
  selectedStepId,
  onSelectStep,
  onAddBranchStep,
  selectedWorkflowRuns,
  expandedRunId,
  attemptsByRun,
  onToggleAttempts,
}: WorkflowBuilderPanelProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="absolute bottom-3 left-3 top-16 z-30 w-[340px] overflow-y-auto rounded-lg border border-zinc-200 bg-white/95 p-3 shadow-lg backdrop-blur">
      <div className="space-y-3">
        <div className="space-y-2 rounded-md border border-zinc-200 bg-zinc-50 p-2">
          <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
            Workflow Library
          </p>
          <Input
            value={workflowQuery}
            onChange={(event) => onWorkflowQueryChange(event.target.value)}
            placeholder="Search workflows"
          />
          <div className="max-h-40 space-y-1 overflow-y-auto pr-1">
            {filteredWorkflows.length > 0 ? (
              filteredWorkflows.map((workflow) => {
                const isSelected = selectedWorkflow === workflow.logical_name;
                return (
                  <div
                    key={workflow.logical_name}
                    className={`rounded-md border px-2 py-2 ${
                      isSelected
                        ? "border-emerald-300 bg-emerald-50"
                        : "border-zinc-200 bg-white"
                    }`}
                  >
                    <p className="text-xs font-semibold text-zinc-900">
                      {workflow.display_name}
                    </p>
                    <p className="font-mono text-[10px] text-zinc-500">
                      {workflow.logical_name}
                    </p>
                    <div className="mt-2 flex gap-1">
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        onClick={() => onLoadWorkflow(workflow)}
                      >
                        Edit
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        onClick={() => onOpenWorkflowHistory(workflow)}
                      >
                        History
                      </Button>
                    </div>
                  </div>
                );
              })
            ) : (
              <p className="text-[11px] text-zinc-500">No matching workflows.</p>
            )}
          </div>
          <Button type="button" size="sm" variant="outline" onClick={onResetBuilder}>
            New Workflow
          </Button>
        </div>

        {workflowWorkspaceMode === "edit" ? (
          <>
            <form className="space-y-3" onSubmit={onSaveWorkflow}>
              <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
                Flow Builder
              </p>
              <Input
                value={logicalName}
                onChange={(event) => onLogicalNameChange(event.target.value)}
                placeholder="logical_name"
                required
              />
              <Input
                value={displayName}
                onChange={(event) => onDisplayNameChange(event.target.value)}
                placeholder="Display name"
                required
              />
              <Input
                value={description}
                onChange={(event) => onDescriptionChange(event.target.value)}
                placeholder="Description"
              />
              <Input
                type="number"
                min={1}
                max={10}
                value={maxAttempts}
                onChange={(event) => onMaxAttemptsChange(event.target.value)}
                required
              />
              <label className="inline-flex items-center gap-2 text-xs text-zinc-700">
                <input
                  type="checkbox"
                  checked={isEnabled}
                  onChange={(event) => onEnabledChange(event.target.checked)}
                />
                enabled
              </label>

              <div className="space-y-2 rounded-md border border-zinc-200 bg-zinc-50 p-2">
                <p className="text-[11px] font-semibold uppercase tracking-wide text-zinc-600">
                  Function Catalog
                </p>
                <Input
                  value={catalogQuery}
                  onChange={(event) => onCatalogQueryChange(event.target.value)}
                  placeholder="Search functions"
                />
                <div className="grid grid-cols-2 gap-2">
                  <Select
                    value={catalogCategory}
                    onChange={(event) =>
                      onCatalogCategoryChange(
                        event.target.value as "all" | FlowTemplateCategory,
                      )
                    }
                  >
                    <option value="all">All</option>
                    <option value="trigger">Trigger</option>
                    <option value="logic">Logic</option>
                    <option value="integration">Integration</option>
                    <option value="data">Data</option>
                    <option value="operations">Operations</option>
                  </Select>

                  <Select
                    value={catalogInsertMode}
                    onChange={(event) =>
                      onCatalogInsertModeChange(event.target.value as CatalogInsertMode)
                    }
                  >
                    <option value="after_selected">After selected</option>
                    <option value="root">Append root</option>
                    <option value="then_selected" disabled={!canInsertIntoConditionBranch}>
                      Condition: yes
                    </option>
                    <option value="else_selected" disabled={!canInsertIntoConditionBranch}>
                      Condition: no
                    </option>
                  </Select>
                </div>

                <div className="max-h-52 space-y-1 overflow-y-auto pr-1">
                  {filteredTemplates.length > 0 ? (
                    filteredTemplates.map((template) => (
                      <button
                        key={template.id}
                        type="button"
                        className="w-full rounded-md border border-zinc-200 bg-white px-2 py-2 text-left transition hover:border-emerald-300"
                        onClick={() => onInsertTemplate(template.id)}
                      >
                        <p className="text-xs font-semibold text-zinc-900">{template.label}</p>
                        <p className="text-[11px] text-zinc-600">{template.description}</p>
                      </button>
                    ))
                  ) : (
                    <p className="text-[11px] text-zinc-500">No matching functions.</p>
                  )}
                </div>

                <div className="flex flex-wrap gap-1">
                  {STEP_LIBRARY.map((entry) => (
                    <Button
                      key={entry.type}
                      type="button"
                      size="sm"
                      variant="outline"
                      onClick={() => onAddRootStep(entry.type)}
                    >
                      + {entry.label}
                    </Button>
                  ))}
                </div>
              </div>
              <Button type="submit" disabled={isSaving}>
                {isSaving ? "Saving..." : "Save Flow"}
              </Button>
            </form>

            <Separator className="my-3" />

            <form className="space-y-2" onSubmit={onExecuteWorkflow}>
              <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
                Test Run
              </p>
              <Select
                value={selectedWorkflow}
                onChange={(event) => onExecutionWorkflowChange(event.target.value)}
              >
                <option value="">Select workflow</option>
                {workflows.map((workflow) => (
                  <option key={workflow.logical_name} value={workflow.logical_name}>
                    {workflow.display_name}
                  </option>
                ))}
              </Select>
              <Textarea
                className="font-mono text-xs"
                value={executePayload}
                onChange={(event) => onExecutePayloadChange(event.target.value)}
                rows={4}
              />
              <Button type="submit" size="sm" variant="outline" disabled={isExecuting}>
                {isExecuting ? "Executing..." : "Execute"}
              </Button>
            </form>

            <Separator className="my-3" />
            <FlowOutlinePanel
              steps={steps}
              selectedStepId={selectedStepId}
              onSelectStep={onSelectStep}
              onAddBranchStep={onAddBranchStep}
            />
          </>
        ) : (
          <RunHistoryPanel
            selectedWorkflow={selectedWorkflow}
            selectedWorkflowRuns={selectedWorkflowRuns}
            expandedRunId={expandedRunId}
            attemptsByRun={attemptsByRun}
            onToggleAttempts={onToggleAttempts}
          />
        )}
      </div>
    </div>
  );
}
