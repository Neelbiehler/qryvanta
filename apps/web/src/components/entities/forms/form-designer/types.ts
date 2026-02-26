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

export type SelectionState =
  | { kind: "tab"; tabIndex: number }
  | { kind: "section"; tabIndex: number; sectionIndex: number }
  | {
      kind: "field";
      tabIndex: number;
      sectionIndex: number;
      fieldIndex: number;
    };
