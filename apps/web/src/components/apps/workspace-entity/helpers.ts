import type { PublishedSchemaResponse } from "@/lib/api";

export function buildInitialValues(
  schema: PublishedSchemaResponse,
): Record<string, unknown> {
  const values: Record<string, unknown> = {};
  for (const field of schema.fields) {
    if (field.default_value !== null) {
      values[field.logical_name] = field.default_value;
      continue;
    }

    if (field.field_type === "boolean") {
      values[field.logical_name] = false;
      continue;
    }

    values[field.logical_name] = "";
  }

  return values;
}

export function resolveConfiguredFields(
  schema: PublishedSchemaResponse,
  configuredFieldLogicalNames: string[],
): PublishedSchemaResponse["fields"] {
  if (configuredFieldLogicalNames.length === 0) {
    return schema.fields;
  }

  const fieldByLogicalName = new Map(
    schema.fields.map((field) => [field.logical_name, field]),
  );

  const configuredFields = configuredFieldLogicalNames
    .map((logicalName) => fieldByLogicalName.get(logicalName))
    .filter((field): field is PublishedSchemaResponse["fields"][number] =>
      Boolean(field),
    );

  return configuredFields.length > 0 ? configuredFields : schema.fields;
}

export function formatValue(value: unknown): string {
  if (value === null || value === undefined || value === "") {
    return "-";
  }

  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean"
  ) {
    return String(value);
  }

  return JSON.stringify(value);
}
