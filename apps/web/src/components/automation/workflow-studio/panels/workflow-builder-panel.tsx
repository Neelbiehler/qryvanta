"use client";

import { useState, type FormEvent } from "react";
import { Play, Search, Settings2 } from "lucide-react";

import type { WorkflowResponse } from "@/lib/api";
import type {
  CatalogInsertMode,
  DraftObjectField,
  DraftWorkflowStep,
  FlowTemplateCategory,
  FlowTemplateId,
  TriggerType,
  WorkflowValidationIssue,
} from "@/components/automation/workflow-studio/model";
import {
  ActionsTab,
  DetailsTab,
  type Tab,
  TabButton,
  type TemplateOption,
  TestTab,
} from "@/components/automation/workflow-studio/panels/workflow-builder-panel-tabs";

type WorkflowBuilderPanelProps = {
  open: boolean;
  onSaveWorkflow: (event: FormEvent<HTMLFormElement>) => void;
  logicalName: string;
  onLogicalNameChange: (value: string) => void;
  displayName: string;
  onDisplayNameChange: (value: string) => void;
  description: string;
  onDescriptionChange: (value: string) => void;
  maxAttempts: string;
  onMaxAttemptsChange: (value: string) => void;
  workflowLifecycleState: WorkflowResponse["lifecycle_state"];
  publishedVersion: number | null;
  catalogQuery: string;
  onCatalogQueryChange: (value: string) => void;
  catalogCategory: "all" | FlowTemplateCategory;
  onCatalogCategoryChange: (value: "all" | FlowTemplateCategory) => void;
  catalogInsertMode: CatalogInsertMode;
  canInsertIntoConditionBranch: boolean;
  filteredTemplates: TemplateOption[];
  onInsertTemplate: (templateId: FlowTemplateId) => void;
  onAddRootStep: (stepType: DraftWorkflowStep["type"]) => void;
  isSaving: boolean;
  isPublishing: boolean;
  isDisabling: boolean;
  onPublishWorkflow: () => void;
  onDisableWorkflow: () => void;
  onExecuteWorkflow: (event: FormEvent<HTMLFormElement>) => void;
  onExecutionWorkflowChange: (workflowLogicalName: string) => void;
  workflows: WorkflowResponse[];
  selectedWorkflow: string;
  selectedWorkflowDefinition: WorkflowResponse | null;
  selectedWorkflowTriggerSchema: { fields: Array<{ logical_name: string; display_name: string; field_type: string }> } | null;
  executePayloadFields: DraftObjectField[];
  onExecutePayloadFieldsChange: (fields: DraftObjectField[]) => void;
  onLoadSuggestedExecutePayload: () => void;
  isExecuting: boolean;
  validationIssues: WorkflowValidationIssue[];
  validationErrorCount: number;
  onFocusValidationIssue: (issue: WorkflowValidationIssue) => void;
};

export function WorkflowBuilderPanel({
  open,
  onSaveWorkflow,
  logicalName,
  onLogicalNameChange,
  displayName,
  onDisplayNameChange,
  description,
  onDescriptionChange,
  maxAttempts,
  onMaxAttemptsChange,
  workflowLifecycleState,
  publishedVersion,
  catalogQuery,
  onCatalogQueryChange,
  catalogCategory,
  onCatalogCategoryChange,
  filteredTemplates,
  onInsertTemplate,
  onAddRootStep,
  isSaving,
  isPublishing,
  isDisabling,
  onPublishWorkflow,
  onDisableWorkflow,
  onExecuteWorkflow,
  onExecutionWorkflowChange,
  workflows,
  selectedWorkflow,
  selectedWorkflowDefinition,
  selectedWorkflowTriggerSchema,
  executePayloadFields,
  onExecutePayloadFieldsChange,
  onLoadSuggestedExecutePayload,
  isExecuting,
  validationIssues,
  validationErrorCount,
  onFocusValidationIssue,
}: WorkflowBuilderPanelProps) {
  const [activeTab, setActiveTab] = useState<Tab>("actions");

  if (!open) return null;

  return (
    <div className="absolute bottom-3 left-3 top-[52px] z-30 flex w-72 flex-col overflow-hidden rounded-xl border border-zinc-200 bg-white shadow-lg">
      <div className="flex shrink-0 border-b border-zinc-200">
        <TabButton active={activeTab === "actions"} onClick={() => setActiveTab("actions")}>
          <Search className="size-3.5" />
          Actions
        </TabButton>
        <TabButton
          active={activeTab === "details"}
          onClick={() => setActiveTab("details")}
          badge={validationErrorCount > 0 ? validationErrorCount : undefined}
        >
          <Settings2 className="size-3.5" />
          Details
        </TabButton>
        <TabButton active={activeTab === "test"} onClick={() => setActiveTab("test")}>
          <Play className="size-3.5" />
          Test
        </TabButton>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {activeTab === "actions" && (
          <ActionsTab
            catalogQuery={catalogQuery}
            onCatalogQueryChange={onCatalogQueryChange}
            catalogCategory={catalogCategory}
            onCatalogCategoryChange={onCatalogCategoryChange}
            filteredTemplates={filteredTemplates}
            onInsertTemplate={onInsertTemplate}
            onAddRootStep={onAddRootStep}
          />
        )}
        {activeTab === "details" && (
          <DetailsTab
            logicalName={logicalName}
            onLogicalNameChange={onLogicalNameChange}
            displayName={displayName}
            onDisplayNameChange={onDisplayNameChange}
            description={description}
            onDescriptionChange={onDescriptionChange}
            maxAttempts={maxAttempts}
            onMaxAttemptsChange={onMaxAttemptsChange}
            workflowLifecycleState={workflowLifecycleState}
            publishedVersion={publishedVersion}
            isSaving={isSaving}
            isPublishing={isPublishing}
            isDisabling={isDisabling}
            validationIssues={validationIssues}
            validationErrorCount={validationErrorCount}
            onFocusValidationIssue={onFocusValidationIssue}
            onSaveWorkflow={onSaveWorkflow}
            onPublishWorkflow={onPublishWorkflow}
            onDisableWorkflow={onDisableWorkflow}
          />
        )}
        {activeTab === "test" && (
          <TestTab
            workflows={workflows}
            selectedWorkflow={selectedWorkflow}
            selectedWorkflowDefinition={selectedWorkflowDefinition}
            selectedWorkflowTriggerSchema={selectedWorkflowTriggerSchema}
            executePayloadFields={executePayloadFields}
            onExecutePayloadFieldsChange={onExecutePayloadFieldsChange}
            onLoadSuggestedExecutePayload={onLoadSuggestedExecutePayload}
            isExecuting={isExecuting}
            onExecuteWorkflow={onExecuteWorkflow}
            onExecutionWorkflowChange={onExecutionWorkflowChange}
          />
        )}
      </div>
    </div>
  );
}
