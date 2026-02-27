import {
  Checkbox,
  Input,
  Label,
  Select,
  Textarea,
} from "@qryvanta/ui";

import type {
  EvaluatedRuleState,
} from "@/components/apps/workspace-entity/business-rules";
import type {
  FormFieldPlacement,
} from "@/components/apps/workspace-entity/metadata-types";
import { RelationFieldSelect } from "@/components/apps/relation-field-select";
import type {
  FieldResponse,
  OptionSetResponse,
} from "@/lib/api";

type FieldControlProps = {
  appLogicalName: string;
  placement: FormFieldPlacement;
  field: FieldResponse | null;
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  optionSets: OptionSetResponse[];
  ruleState: EvaluatedRuleState;
  canEdit?: boolean;
  fieldIdPrefix?: string;
  prettyJsonObjects?: boolean;
  jsonRows?: number;
};

export function FieldControl({
  appLogicalName,
  placement,
  field,
  formValues,
  onFieldValueChange,
  optionSets,
  ruleState,
  canEdit = true,
  fieldIdPrefix = "field",
  prettyJsonObjects = false,
  jsonRows,
}: FieldControlProps) {
  if (!field) {
    return (
      <div className="text-xs text-zinc-400">
        Unknown field: {placement.field_logical_name}
      </div>
    );
  }

  const fieldId = `${fieldIdPrefix}_${field.logical_name}`;
  const value = formValues[field.logical_name];
  const displayLabel = placement.label_override ?? field.display_name;
  const isRequired =
    ruleState.requiredOverrides.get(field.logical_name) ??
    (placement.required_override !== null
      ? placement.required_override
      : field.is_required);
  const isSystemIdentifier = field.logical_name === "record_id";
  const isReadOnly =
    (ruleState.readOnlyOverrides.get(field.logical_name) ?? placement.read_only) ||
    !canEdit ||
    isSystemIdentifier;

  if (field.option_set_logical_name) {
    const optionSet = optionSets.find(
      (optionSetItem) => optionSetItem.logical_name === field.option_set_logical_name,
    );

    if (optionSet) {
      return (
        <div className="space-y-2">
          <Label htmlFor={fieldId}>
            {displayLabel}
            {isRequired ? <span className="text-red-500"> *</span> : null}
          </Label>
          <Select
            id={fieldId}
            value={String(value ?? "")}
            onChange={(event) => {
              const numericValue = Number(event.target.value);
              onFieldValueChange(
                field.logical_name,
                Number.isNaN(numericValue) ? event.target.value : numericValue,
              );
            }}
            disabled={isReadOnly}
            required={isRequired}
          >
            <option value="">-- Select --</option>
            {optionSet.options
              .toSorted((left, right) => left.position - right.position)
              .map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
          </Select>
        </div>
      );
    }
  }

  if (field.field_type === "relation" && field.relation_target_entity) {
    return (
      <div className="space-y-2">
        <Label htmlFor={fieldId}>
          {displayLabel}
          {isRequired ? <span className="text-red-500"> *</span> : null}
        </Label>
        <RelationFieldSelect
          appLogicalName={appLogicalName}
          entityLogicalName={field.relation_target_entity}
          fieldId={fieldId}
          value={value}
          disabled={isReadOnly}
          required={isRequired}
          onChange={(nextValue) => onFieldValueChange(field.logical_name, nextValue)}
        />
      </div>
    );
  }

  if (field.field_type === "boolean") {
    return (
      <div className="space-y-2">
        <Label htmlFor={fieldId}>{displayLabel}</Label>
        <label className="inline-flex items-center gap-2 text-sm text-zinc-700">
          <Checkbox
            id={fieldId}
            checked={Boolean(value)}
            onChange={(event) => onFieldValueChange(field.logical_name, event.target.checked)}
            disabled={isReadOnly}
          />
          {displayLabel}
        </label>
      </div>
    );
  }

  if (field.field_type === "json") {
    const displayValue =
      prettyJsonObjects && typeof value === "object" && value !== null
        ? JSON.stringify(value, null, 2)
        : String(value ?? "");

    return (
      <div className="space-y-2">
        <Label htmlFor={fieldId}>
          {displayLabel}
          {isRequired ? <span className="text-red-500"> *</span> : null}
        </Label>
        <Textarea
          id={fieldId}
          className="font-mono text-xs"
          value={displayValue}
          onChange={(event) => onFieldValueChange(field.logical_name, event.target.value)}
          placeholder='{"value":"example"}'
          spellCheck={false}
          autoComplete="off"
          readOnly={isReadOnly}
          required={isRequired}
          rows={jsonRows}
        />
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <Label htmlFor={fieldId}>
        {displayLabel}
        {isRequired ? <span className="text-red-500"> *</span> : null}
      </Label>
      <Input
        id={fieldId}
        type={
          field.field_type === "number"
            ? "number"
            : field.field_type === "date"
              ? "date"
              : field.field_type === "datetime"
                ? "datetime-local"
                : "text"
        }
        value={String(value ?? "")}
        onChange={(event) => onFieldValueChange(field.logical_name, event.target.value)}
        required={isRequired}
        readOnly={isReadOnly}
      />
    </div>
  );
}
