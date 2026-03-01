import { Input, Label, Select } from "@qryvanta/ui";

import {
  RUNTIME_TRIGGER_ENTITY_PRESETS,
  SCHEDULE_TRIGGER_KEY_PRESETS,
  TRIGGER_OPTIONS,
  type TriggerType,
} from "@/components/automation/workflow-studio/model";

type TriggerConfigPanelProps = {
  triggerType: TriggerType;
  triggerEntityLogicalName: string;
  runtimeEntityOptions?: Array<{ value: string; label: string }>;
  onTriggerTypeChange: (value: TriggerType) => void;
  onTriggerEntityChange: (value: string) => void;
};

export function TriggerConfigPanel({
  triggerType,
  triggerEntityLogicalName,
  runtimeEntityOptions = [],
  onTriggerTypeChange,
  onTriggerEntityChange,
}: TriggerConfigPanelProps) {
  const isRuntimeEntityTrigger =
    triggerType === "runtime_record_created" ||
    triggerType === "runtime_record_updated" ||
    triggerType === "runtime_record_deleted";
  const isScheduleTrigger = triggerType === "schedule_tick";

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
              {option.label}
            </option>
          ))}
        </Select>
      </div>
      <div className="space-y-2">
        <Label htmlFor="workflow_trigger_entity_preset">Common Event Source</Label>
        <Select
          id="workflow_trigger_entity_preset"
          value={
            [...RUNTIME_TRIGGER_ENTITY_PRESETS, ...runtimeEntityOptions].some(
              (preset) => preset.value === triggerEntityLogicalName,
            )
              ? triggerEntityLogicalName
              : ""
          }
          onChange={(event) => onTriggerEntityChange(event.target.value)}
          disabled={!isRuntimeEntityTrigger}
        >
          <option value="">Custom entity</option>
          {runtimeEntityOptions.map((preset) => (
            <option key={preset.value} value={preset.value}>
              {preset.label}
            </option>
          ))}
          {RUNTIME_TRIGGER_ENTITY_PRESETS.map((preset) => (
            <option key={preset.value} value={preset.value}>
              {preset.label}
            </option>
          ))}
        </Select>
      </div>
      <div className="space-y-2">
        <Label htmlFor="workflow_trigger_entity">
          {isScheduleTrigger ? "Schedule Key" : "Trigger Entity"}
        </Label>
        {isScheduleTrigger ? (
          <Select
            id="workflow_trigger_schedule_key_preset"
            value={
              SCHEDULE_TRIGGER_KEY_PRESETS.some(
                (preset) => preset.value === triggerEntityLogicalName,
              )
                ? triggerEntityLogicalName
                : ""
            }
            onChange={(event) => onTriggerEntityChange(event.target.value)}
          >
            <option value="">Custom schedule key</option>
            {SCHEDULE_TRIGGER_KEY_PRESETS.map((preset) => (
              <option key={preset.value} value={preset.value}>
                {preset.label}
              </option>
            ))}
          </Select>
        ) : null}
        <Input
          id="workflow_trigger_entity"
          value={triggerEntityLogicalName}
          onChange={(event) => onTriggerEntityChange(event.target.value)}
          placeholder={isScheduleTrigger ? "hourly" : "contact"}
          list="workflow_trigger_entity_suggestions"
          disabled={!isRuntimeEntityTrigger && !isScheduleTrigger}
        />
        <datalist id="workflow_trigger_entity_suggestions">
          {runtimeEntityOptions.map((preset) => (
            <option key={preset.value} value={preset.value} />
          ))}
          {RUNTIME_TRIGGER_ENTITY_PRESETS.map((preset) => (
            <option key={preset.value} value={preset.value} />
          ))}
        </datalist>
      </div>
    </>
  );
}
