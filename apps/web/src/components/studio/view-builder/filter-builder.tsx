"use client";

import { Button, Input, Label, Select } from "@qryvanta/ui";

import type { ViewEditorState } from "@/components/studio/hooks/use-view-editor-state";
import type { FieldResponse } from "@/lib/api";

type FilterBuilderProps = {
  fields: FieldResponse[];
  viewEditor: ViewEditorState;
};

export function FilterBuilder({ fields, viewEditor }: FilterBuilderProps) {
  return (
    <div className="space-y-2 rounded-lg border border-zinc-200 bg-white p-3">
      <div className="flex items-center justify-between">
        <Label htmlFor="studio_view_filter_mode" className="text-xs">
          Filters
        </Label>
        <Button
          type="button"
          size="sm"
          variant="outline"
          onClick={() => viewEditor.addFilterRule(fields[0]?.logical_name ?? "")}
        >
          Add Rule
        </Button>
      </div>

      <Select
        id="studio_view_filter_mode"
        value={viewEditor.filterGroup?.logical_mode ?? "and"}
        onChange={(event) =>
          viewEditor.setFilterGroup({
            logical_mode: event.target.value === "or" ? "or" : "and",
            conditions: viewEditor.filterGroup?.conditions ?? [],
          })
        }
        className="h-8 text-xs"
      >
        <option value="and">AND</option>
        <option value="or">OR</option>
      </Select>

      {(viewEditor.filterGroup?.conditions ?? []).map((condition, index) => (
        <div key={`${condition.field_logical_name}-${index}`} className="grid grid-cols-4 gap-1">
          {(() => {
            const selectedField = fields.find(
              (field) => field.logical_name === condition.field_logical_name,
            );
            const operators = getOperatorsForField(selectedField?.field_type);
            return (
              <>
          <Select
            value={condition.field_logical_name}
            onChange={(event) =>
              viewEditor.updateFilterRule(index, {
                field_logical_name: event.target.value,
                operator:
                  getOperatorsForField(
                    fields.find((field) => field.logical_name === event.target.value)?.field_type,
                  )[0] ?? "eq",
              })
            }
            className="h-8 text-xs"
          >
            {fields.map((field) => (
              <option key={field.logical_name} value={field.logical_name}>
                {field.display_name}
              </option>
            ))}
          </Select>
          <Select
            value={condition.operator}
            onChange={(event) =>
              viewEditor.updateFilterRule(index, {
                operator: event.target.value as
                  | "eq"
                  | "neq"
                  | "gt"
                  | "gte"
                  | "lt"
                  | "lte"
                  | "contains"
                  | "in",
              })
            }
            className="h-8 text-xs"
          >
            {operators.map((operator) => (
              <option key={`${condition.field_logical_name}-${operator}`} value={operator}>
                {operator}
              </option>
            ))}
          </Select>
          <Input
            value={condition.value}
            onChange={(event) =>
              viewEditor.updateFilterRule(index, { value: event.target.value })
            }
            className="h-8 text-xs"
            placeholder={resolveValuePlaceholder(selectedField?.field_type)}
          />
          <Button
            type="button"
            size="sm"
            variant="ghost"
            onClick={() => viewEditor.removeFilterRule(index)}
          >
            Remove
          </Button>
              </>
            );
          })()}
        </div>
      ))}
    </div>
  );
}

function getOperatorsForField(fieldType: string | undefined): Array<
  "eq" | "neq" | "gt" | "gte" | "lt" | "lte" | "contains" | "in"
> {
  if (!fieldType) return ["eq", "neq", "contains"];

  if (fieldType === "date" || fieldType === "datetime") {
    return ["eq", "neq", "gte", "lte", "gt", "lt"];
  }

  if (fieldType === "number") {
    return ["eq", "neq", "gt", "gte", "lt", "lte", "in"];
  }

  if (fieldType === "boolean") {
    return ["eq", "neq"];
  }

  return ["eq", "neq", "contains", "in"];
}

function resolveValuePlaceholder(fieldType: string | undefined): string {
  if (fieldType === "date") return "YYYY-MM-DD";
  if (fieldType === "datetime") return "YYYY-MM-DDTHH:mm";
  if (fieldType === "number") return "e.g. 100";
  if (fieldType === "boolean") return "true / false";
  return "value";
}
