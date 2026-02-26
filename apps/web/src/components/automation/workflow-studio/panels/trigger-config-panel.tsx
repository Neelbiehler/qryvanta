import { Input, Label, Select } from "@qryvanta/ui";

import {
  TRIGGER_OPTIONS,
  type TriggerType,
} from "@/components/automation/workflow-studio/model";

type TriggerConfigPanelProps = {
  triggerType: TriggerType;
  triggerEntityLogicalName: string;
  onTriggerTypeChange: (value: TriggerType) => void;
  onTriggerEntityChange: (value: string) => void;
};

export function TriggerConfigPanel({
  triggerType,
  triggerEntityLogicalName,
  onTriggerTypeChange,
  onTriggerEntityChange,
}: TriggerConfigPanelProps) {
  return (
    <>
      <div className="space-y-2">
        <Label htmlFor="workflow_trigger_type">Trigger Type</Label>
        <Select
          id="workflow_trigger_type"
          value={triggerType}
          onChange={(event) => onTriggerTypeChange(event.target.value as TriggerType)}
        >
          {TRIGGER_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.value}
            </option>
          ))}
        </Select>
      </div>
      <div className="space-y-2">
        <Label htmlFor="workflow_trigger_entity">Trigger Entity</Label>
        <Input
          id="workflow_trigger_entity"
          value={triggerEntityLogicalName}
          onChange={(event) => onTriggerEntityChange(event.target.value)}
          placeholder="contact"
          disabled={triggerType !== "runtime_record_created"}
        />
      </div>
    </>
  );
}
