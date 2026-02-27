"use client";

import { Checkbox, Input, Label, Select, Textarea } from "@qryvanta/ui";

import type { FormFieldPlacement } from "@/components/studio/types";
import type { FieldResponse, OptionSetResponse } from "@/lib/api";

type WysiwygFieldRendererProps = {
  placement: FormFieldPlacement;
  field: FieldResponse | null;
  sampleValue: unknown;
  optionSets: OptionSetResponse[];
  previewMode: boolean;
};

export function WysiwygFieldRenderer({
  placement,
  field,
  sampleValue,
  optionSets,
  previewMode,
}: WysiwygFieldRendererProps) {
  if (!field) {
    return (
      <div className="rounded-md border border-dashed border-zinc-300 bg-zinc-50 px-2 py-2 text-xs text-zinc-500">
        Unknown field: {placement.field_logical_name}
      </div>
    );
  }

  const label = placement.label_override?.trim() || field.display_name;
  const required =
    placement.required_override !== null
      ? placement.required_override
      : field.is_required;
  const readOnly = !previewMode || placement.read_only;
  const controlId = `studio_field_${field.logical_name}`;
  const currentValue =
    sampleValue === undefined || sampleValue === null ? "" : sampleValue;
  const noop = () => undefined;

  if (field.field_type === "boolean") {
    return (
      <div className="space-y-1.5">
        <Label htmlFor={controlId} className="text-[11px] text-zinc-700">
          {label}
        </Label>
        <label className="inline-flex items-center gap-2 text-xs text-zinc-700">
          <Checkbox
            id={controlId}
            checked={Boolean(currentValue)}
            disabled={readOnly}
            onChange={noop}
          />
          {label}
        </label>
      </div>
    );
  }

  if (field.option_set_logical_name) {
    const optionSet = optionSets.find(
      (candidate) => candidate.logical_name === field.option_set_logical_name,
    );
    if (optionSet) {
      return (
        <div className="space-y-1.5">
          <Label htmlFor={controlId} className="text-[11px] text-zinc-700">
            {label}
            {required ? <span className="text-red-500"> *</span> : null}
          </Label>
          <Select
            id={controlId}
            value={String(currentValue)}
            disabled={readOnly}
            onChange={noop}
          >
            <option value="">-- Select --</option>
            {optionSet.options
              .slice()
              .sort((left, right) => left.position - right.position)
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

  if (field.field_type === "json") {
    const displayJson =
      typeof currentValue === "object" && currentValue !== null
        ? JSON.stringify(currentValue, null, 2)
        : String(currentValue);

    return (
      <div className="space-y-1.5">
        <Label htmlFor={controlId} className="text-[11px] text-zinc-700">
          {label}
          {required ? <span className="text-red-500"> *</span> : null}
        </Label>
        <Textarea
          id={controlId}
          className="min-h-24 font-mono text-xs"
          readOnly={readOnly}
          value={displayJson}
          onChange={noop}
        />
      </div>
    );
  }

  return (
    <div className="space-y-1.5">
      <Label htmlFor={controlId} className="text-[11px] text-zinc-700">
        {label}
        {required ? <span className="text-red-500"> *</span> : null}
      </Label>
      <Input
        id={controlId}
        readOnly={readOnly}
        required={required}
        value={String(currentValue)}
        onChange={noop}
        type={
          field.field_type === "number"
            ? "number"
            : field.field_type === "date"
              ? "date"
              : field.field_type === "datetime"
                ? "datetime-local"
                : "text"
        }
      />
    </div>
  );
}
