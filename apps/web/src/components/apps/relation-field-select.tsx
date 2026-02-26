"use client";

import { useEffect, useMemo, useState } from "react";

import { Select } from "@qryvanta/ui";

import {
  apiFetch,
  type FieldResponse,
  type PublishedSchemaResponse,
  type RuntimeRecordResponse,
} from "@/lib/api";

type RelationFieldSelectProps = {
  appLogicalName: string;
  entityLogicalName: string;
  fieldId: string;
  value: unknown;
  disabled: boolean;
  required: boolean;
  onChange: (value: string) => void;
};

type RelationOption = {
  recordId: string;
  label: string;
};

function pickDisplayField(fields: FieldResponse[]): FieldResponse | null {
  const nameField = fields.find((field) => field.logical_name === "name");
  if (nameField) {
    return nameField;
  }

  const firstTextField = fields.find((field) => field.field_type === "text");
  if (firstTextField) {
    return firstTextField;
  }

  return fields[0] ?? null;
}

export function RelationFieldSelect({
  appLogicalName,
  entityLogicalName,
  fieldId,
  value,
  disabled,
  required,
  onChange,
}: RelationFieldSelectProps) {
  const [options, setOptions] = useState<RelationOption[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    let isMounted = true;

    async function loadRelationOptions() {
      setIsLoading(true);

      try {
        const [schemaResponse, recordsResponse] = await Promise.all([
          apiFetch(
            `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/schema`,
          ),
          apiFetch(
            `/api/workspace/apps/${appLogicalName}/entities/${entityLogicalName}/records?limit=100&offset=0`,
          ),
        ]);

        if (!schemaResponse.ok || !recordsResponse.ok) {
          if (isMounted) {
            setOptions([]);
          }
          return;
        }

        const schema = (await schemaResponse.json()) as PublishedSchemaResponse;
        const records = (await recordsResponse.json()) as RuntimeRecordResponse[];
        const displayField = pickDisplayField(schema.fields);

        const nextOptions = records.map((record) => {
          const displayValue = displayField
            ? record.data[displayField.logical_name]
            : null;
          const valueText =
            displayValue === null || displayValue === undefined
              ? ""
              : String(displayValue);

          return {
            recordId: record.record_id,
            label: valueText.trim().length > 0 ? valueText : record.record_id,
          } satisfies RelationOption;
        });

        if (isMounted) {
          setOptions(nextOptions);
        }
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    }

    void loadRelationOptions();

    return () => {
      isMounted = false;
    };
  }, [appLogicalName, entityLogicalName]);

  const selectedValue = useMemo(() => String(value ?? ""), [value]);

  return (
    <Select
      id={fieldId}
      value={selectedValue}
      onChange={(event) => onChange(event.target.value)}
      disabled={disabled || isLoading}
      required={required}
    >
      <option value="">{isLoading ? "Loading records..." : "-- Select --"}</option>
      {options.map((option) => (
        <option key={option.recordId} value={option.recordId}>
          {option.label} ({option.recordId})
        </option>
      ))}
    </Select>
  );
}
