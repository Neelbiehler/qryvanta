import { Button, Input, Label, Select, Textarea } from "@qryvanta/ui";

import { TriggerConfigPanel } from "@/components/automation/workflow-studio/panels/trigger-config-panel";
import {
  CONDITION_OPERATORS,
  type DraftWorkflowStep,
  type InspectorNode,
  type TriggerType,
} from "@/components/automation/workflow-studio/model";
import type { WorkflowConditionOperatorDto } from "@/lib/api";

type WorkflowInspectorPanelProps = {
  open: boolean;
  inspectorNode: InspectorNode;
  selectedStep: DraftWorkflowStep | null;
  triggerType: TriggerType;
  triggerEntityLogicalName: string;
  onTriggerTypeChange: (value: TriggerType) => void;
  onTriggerEntityChange: (value: string) => void;
  onUpdateSelectedStep: (
    updater: (step: DraftWorkflowStep) => DraftWorkflowStep,
  ) => void;
  onRemoveSelectedStep: () => void;
};

export function WorkflowInspectorPanel({
  open,
  inspectorNode,
  selectedStep,
  triggerType,
  triggerEntityLogicalName,
  onTriggerTypeChange,
  onTriggerEntityChange,
  onUpdateSelectedStep,
  onRemoveSelectedStep,
}: WorkflowInspectorPanelProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="absolute bottom-3 right-3 top-16 z-30 w-[360px] overflow-y-auto rounded-lg border border-zinc-200 bg-white/95 p-3 shadow-lg backdrop-blur">
      <p className="text-xs font-semibold uppercase tracking-wide text-zinc-600">
        Inspector {inspectorNode === "trigger" ? "Trigger" : "Step"}
      </p>
      <div className="mt-3 space-y-3">
        {inspectorNode === "trigger" ? (
          <TriggerConfigPanel
            triggerType={triggerType}
            triggerEntityLogicalName={triggerEntityLogicalName}
            onTriggerTypeChange={onTriggerTypeChange}
            onTriggerEntityChange={onTriggerEntityChange}
          />
        ) : selectedStep ? (
          <>
            {selectedStep.type === "log_message" ? (
              <div className="space-y-2">
                <Label htmlFor="workflow_step_message">Message</Label>
                <Input
                  id="workflow_step_message"
                  value={selectedStep.message}
                  onChange={(event) =>
                    onUpdateSelectedStep((step) =>
                      step.type === "log_message"
                        ? { ...step, message: event.target.value }
                        : step,
                    )
                  }
                />
              </div>
            ) : null}

            {selectedStep.type === "create_runtime_record" ? (
              <>
                <div className="space-y-2">
                  <Label htmlFor="workflow_step_entity">Entity Logical Name</Label>
                  <Input
                    id="workflow_step_entity"
                    value={selectedStep.entityLogicalName}
                    onChange={(event) =>
                      onUpdateSelectedStep((step) =>
                        step.type === "create_runtime_record"
                          ? { ...step, entityLogicalName: event.target.value }
                          : step,
                      )
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="workflow_step_data">Data JSON</Label>
                  <Textarea
                    id="workflow_step_data"
                    className="font-mono text-xs"
                    rows={8}
                    value={selectedStep.dataJson}
                    onChange={(event) =>
                      onUpdateSelectedStep((step) =>
                        step.type === "create_runtime_record"
                          ? { ...step, dataJson: event.target.value }
                          : step,
                      )
                    }
                  />
                </div>
              </>
            ) : null}

            {selectedStep.type === "condition" ? (
              <>
                <Input
                  value={selectedStep.fieldPath}
                  onChange={(event) =>
                    onUpdateSelectedStep((step) =>
                      step.type === "condition"
                        ? { ...step, fieldPath: event.target.value }
                        : step,
                    )
                  }
                  placeholder="field path"
                />
                <Select
                  value={selectedStep.operator}
                  onChange={(event) =>
                    onUpdateSelectedStep((step) =>
                      step.type === "condition"
                        ? {
                            ...step,
                            operator: event.target.value as WorkflowConditionOperatorDto,
                          }
                        : step,
                    )
                  }
                >
                  {CONDITION_OPERATORS.map((operator) => (
                    <option key={operator} value={operator}>
                      {operator}
                    </option>
                  ))}
                </Select>
                <Input
                  value={selectedStep.valueJson}
                  disabled={selectedStep.operator === "exists"}
                  onChange={(event) =>
                    onUpdateSelectedStep((step) =>
                      step.type === "condition"
                        ? { ...step, valueJson: event.target.value }
                        : step,
                    )
                  }
                  placeholder='"open"'
                />
                <div className="grid grid-cols-2 gap-2">
                  <Input
                    value={selectedStep.thenLabel}
                    onChange={(event) =>
                      onUpdateSelectedStep((step) =>
                        step.type === "condition"
                          ? { ...step, thenLabel: event.target.value }
                          : step,
                      )
                    }
                    placeholder="Yes label"
                  />
                  <Input
                    value={selectedStep.elseLabel}
                    onChange={(event) =>
                      onUpdateSelectedStep((step) =>
                        step.type === "condition"
                          ? { ...step, elseLabel: event.target.value }
                          : step,
                      )
                    }
                    placeholder="No label"
                  />
                </div>
              </>
            ) : null}

            <Button type="button" size="sm" variant="outline" onClick={onRemoveSelectedStep}>
              Remove Selected Step
            </Button>
          </>
        ) : (
          <p className="text-sm text-zinc-500">Select any node on canvas to edit.</p>
        )}
      </div>
    </div>
  );
}
