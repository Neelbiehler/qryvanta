/**
 * Local TypeScript interfaces matching the serialized Rust domain types
 * for FormDefinition and ViewDefinition inner structures.
 *
 * The generated `FormResponse.tabs` and `ViewResponse.columns` are typed as
 * `unknown[]` because the Rust API serialises them via `serde_json::Value`.
 * These interfaces provide the actual runtime shapes so the frontend can
 * consume them in a type-safe way.
 */

// ---------------------------------------------------------------------------
// Form types (mirrors crates/domain/src/form.rs serialised output)
// ---------------------------------------------------------------------------

export type FormFieldPlacement = {
  field_logical_name: string;
  column: number;
  position: number;
  visible: boolean;
  read_only: boolean;
  required_override: boolean | null;
  label_override: string | null;
};

export type FormSubgrid = {
  logical_name: string;
  display_name: string;
  target_entity_logical_name: string;
  relation_field_logical_name: string;
  position: number;
  columns: string[];
};

export type FormSection = {
  logical_name: string;
  display_name: string;
  position: number;
  visible: boolean;
  /** Number of layout columns (1, 2, or 3). */
  columns: number;
  fields: FormFieldPlacement[];
  subgrids: FormSubgrid[];
};

export type FormTab = {
  logical_name: string;
  display_name: string;
  position: number;
  visible: boolean;
  sections: FormSection[];
};

// ---------------------------------------------------------------------------
// View types (mirrors crates/domain/src/view.rs serialised output)
// ---------------------------------------------------------------------------

export type ViewColumn = {
  field_logical_name: string;
  position: number;
  width: number | null;
  label_override: string | null;
};

export type ViewSort = {
  field_logical_name: string;
  direction: "asc" | "desc";
};

export type ViewFilterCondition = {
  field_logical_name: string;
  operator: "eq" | "neq" | "gt" | "gte" | "lt" | "lte" | "contains" | "in";
  value: unknown;
};

export type ViewFilterGroup = {
  logical_mode: "and" | "or";
  conditions: ViewFilterCondition[];
};

// ---------------------------------------------------------------------------
// Parsed wrappers for type-safe consumption
// ---------------------------------------------------------------------------

export type ParsedFormResponse = {
  entity_logical_name: string;
  logical_name: string;
  display_name: string;
  form_type: string;
  tabs: FormTab[];
  header_fields: string[];
};

export type ParsedViewResponse = {
  entity_logical_name: string;
  logical_name: string;
  display_name: string;
  view_type: string;
  columns: ViewColumn[];
  default_sort: ViewSort | null;
  filter_criteria: ViewFilterGroup | null;
  is_default: boolean;
};
