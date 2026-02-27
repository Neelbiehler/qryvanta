// ---------------------------------------------------------------------------
// Studio-wide types
// ---------------------------------------------------------------------------

// Re-export form structure types so the studio has a single import source.
// These are identical to the entity-level form designer types.

export type FormTypeValue = "main" | "quick_create" | "quick_view";

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

export type FormSelectionState =
  | { kind: "tab"; tabIndex: number }
  | { kind: "section"; tabIndex: number; sectionIndex: number }
  | {
      kind: "field";
      tabIndex: number;
      sectionIndex: number;
      fieldIndex: number;
    };

// ---------------------------------------------------------------------------
// Studio selection — what is currently open in the central canvas
// ---------------------------------------------------------------------------

export type StudioSelection =
  | { kind: "overview" }
  | { kind: "sitemap" }
  | { kind: "security" }
  | { kind: "publish" }
  | { kind: "form"; entityLogicalName: string; formLogicalName: string }
  | { kind: "view"; entityLogicalName: string; viewLogicalName: string }
  | {
      kind: "business-rule";
      entityLogicalName: string;
      ruleLogicalName: string;
    };

// ---------------------------------------------------------------------------
// Entity tree node shapes (for the solution explorer sidebar)
// ---------------------------------------------------------------------------

export type EntityTreeNode = {
  logicalName: string;
  displayName: string;
  icon?: string;
  forms: { logicalName: string; displayName: string; formType: string }[];
  views: { logicalName: string; displayName: string; viewType: string }[];
  businessRules: { logicalName: string; displayName: string }[];
};

export type SortDirection = "asc" | "desc";
export type LogicalMode = "and" | "or";
export type FilterOperator =
  | "eq"
  | "neq"
  | "gt"
  | "gte"
  | "lt"
  | "lte"
  | "contains"
  | "in";

export type ViewColumn = {
  field_logical_name: string;
  position: number;
  width: number | null;
  label_override: string | null;
};

export type ViewSort = {
  field_logical_name: string;
  direction: SortDirection;
};

export type ViewFilterCondition = {
  field_logical_name: string;
  operator: FilterOperator;
  value: string;
};

export type ViewFilterGroup = {
  logical_mode: LogicalMode;
  conditions: ViewFilterCondition[];
};
