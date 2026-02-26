import type {
  AppSitemapResponse,
  AppSitemapSubAreaDto,
  AppSitemapTargetDto,
  FieldResponse,
  FormResponse,
  OptionSetResponse,
  PublishedSchemaResponse,
  ViewResponse,
} from "@/lib/api";

import type {
  FormTab,
  ParsedFormResponse,
  ParsedViewResponse,
  ViewColumn,
  ViewFilterGroup,
  ViewSort,
} from "./metadata-types";

// ---------------------------------------------------------------------------
// Initial form values
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Field resolution from flat field-name lists (legacy binding approach)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Value formatting
// ---------------------------------------------------------------------------

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

/**
 * Format a field value with option-set awareness.
 * When an option-set is configured for the field, display the label instead
 * of the raw numeric value.
 */
export function formatFieldValue(
  value: unknown,
  field: FieldResponse,
  optionSets: OptionSetResponse[],
): string {
  if (value === null || value === undefined || value === "") {
    return "-";
  }

  if (field.option_set_logical_name) {
    const optionSet = optionSets.find(
      (os) => os.logical_name === field.option_set_logical_name,
    );
    if (optionSet) {
      const option = optionSet.options.find(
        (opt) => opt.value === value || String(opt.value) === String(value),
      );
      if (option) {
        return option.label;
      }
    }
  }

  return formatValue(value);
}

// ---------------------------------------------------------------------------
// Parsing FormResponse -> ParsedFormResponse
// ---------------------------------------------------------------------------

export function parseFormResponse(form: FormResponse): ParsedFormResponse {
  const tabs = (form.tabs as FormTab[])
    .map((tab) => ({
      ...tab,
      sections: [...tab.sections]
        .map((section) => ({
          ...section,
          fields: [...section.fields].sort((a, b) => a.position - b.position),
          subgrids: [...(section.subgrids ?? [])].sort(
            (a, b) => a.position - b.position,
          ),
        }))
        .sort((a, b) => a.position - b.position),
    }))
    .sort((a, b) => a.position - b.position);

  return {
    entity_logical_name: form.entity_logical_name,
    logical_name: form.logical_name,
    display_name: form.display_name,
    form_type: form.form_type,
    tabs,
    header_fields: form.header_fields,
  };
}

// ---------------------------------------------------------------------------
// Parsing ViewResponse -> ParsedViewResponse
// ---------------------------------------------------------------------------

export function parseViewResponse(view: ViewResponse): ParsedViewResponse {
  const columns = (view.columns as ViewColumn[]).sort(
    (a, b) => a.position - b.position,
  );

  return {
    entity_logical_name: view.entity_logical_name,
    logical_name: view.logical_name,
    display_name: view.display_name,
    view_type: view.view_type,
    columns,
    default_sort: (view.default_sort as ViewSort | null) ?? null,
    filter_criteria:
      (view.filter_criteria as ViewFilterGroup | null) ?? null,
    is_default: view.is_default,
  };
}

// ---------------------------------------------------------------------------
// Build a field map from schema for quick lookups
// ---------------------------------------------------------------------------

export function buildFieldMap(
  schema: PublishedSchemaResponse,
): Map<string, FieldResponse> {
  return new Map(schema.fields.map((field) => [field.logical_name, field]));
}

// ---------------------------------------------------------------------------
// Build an option-set map from schema for quick lookups
// ---------------------------------------------------------------------------

export function buildOptionSetMap(
  schema: PublishedSchemaResponse,
): Map<string, OptionSetResponse> {
  return new Map(schema.option_sets.map((os) => [os.logical_name, os]));
}

// ---------------------------------------------------------------------------
// Flatten sitemap tree into navigation items
// ---------------------------------------------------------------------------

/**
 * A flat navigation item derived from the sitemap tree structure.
 * Only entity-targeting sub-areas are included.
 */
export type SitemapNavigationItem = {
  entity_logical_name: string;
  display_name: string;
  position: number;
  icon: string | null;
  default_form: string | null;
  default_view: string | null;
};

export type SitemapDashboardNavigationItem = {
  dashboard_logical_name: string;
  display_name: string;
  position: number;
  icon: string | null;
};

/**
 * Flatten the hierarchical `AppSitemapResponse` into a sorted list of
 * entity navigation items.  Non-entity targets (dashboards, custom pages)
 * are excluded.
 */
export function flattenSitemapToNavigation(
  sitemap: AppSitemapResponse,
): SitemapNavigationItem[] {
  const items: SitemapNavigationItem[] = [];

  const sortedAreas = [...sitemap.areas].sort((a, b) => a.position - b.position);

  for (const area of sortedAreas) {
    const sortedGroups = [...area.groups].sort((a, b) => a.position - b.position);

    for (const group of sortedGroups) {
      const sortedSubAreas = [...group.sub_areas].sort(
        (a, b) => a.position - b.position,
      );

      for (const subArea of sortedSubAreas) {
        if (subArea.target.type === "entity") {
          items.push({
            entity_logical_name: subArea.target.entity_logical_name,
            display_name: subArea.display_name,
            position: subArea.position,
            icon: subArea.icon,
            default_form: subArea.target.default_form ?? null,
            default_view: subArea.target.default_view ?? null,
          });
        }
      }
    }
  }

  return items;
}

/**
 * Flatten the hierarchical `AppSitemapResponse` into a sorted list of
 * dashboard navigation items.
 */
export function flattenSitemapToDashboardNavigation(
  sitemap: AppSitemapResponse,
): SitemapDashboardNavigationItem[] {
  const items: SitemapDashboardNavigationItem[] = [];

  const sortedAreas = [...sitemap.areas].sort((a, b) => a.position - b.position);

  for (const area of sortedAreas) {
    const sortedGroups = [...area.groups].sort((a, b) => a.position - b.position);

    for (const group of sortedGroups) {
      const sortedSubAreas = [...group.sub_areas].sort(
        (a, b) => a.position - b.position,
      );

      for (const subArea of sortedSubAreas) {
        if (subArea.target.type === "dashboard") {
          items.push({
            dashboard_logical_name: subArea.target.dashboard_logical_name,
            display_name: subArea.display_name,
            position: subArea.position,
            icon: subArea.icon,
          });
        }
      }
    }
  }

  return items;
}
