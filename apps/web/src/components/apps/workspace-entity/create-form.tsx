import { type FormEvent } from "react";

import { Button, Notice } from "@qryvanta/ui";

import type {
  EvaluatedRuleState,
} from "@/components/apps/workspace-entity/business-rules";
import { FieldControl } from "@/components/shared/field-control";
import type {
  FieldResponse,
  OptionSetResponse,
  PublishedSchemaResponse,
} from "@/lib/api";
import type {
  FormFieldPlacement,
  FormSection,
  FormTab,
  ParsedFormResponse,
} from "@/components/apps/workspace-entity/metadata-types";

type MetadataDrivenCreateFormProps = {
  activeForm: ParsedFormResponse | null;
  appLogicalName: string;
  canCreate: boolean;
  entityDisplayName: string;
  fieldMap: Map<string, FieldResponse>;
  formValues: Record<string, unknown>;
  isSaving: boolean;
  optionSets: OptionSetResponse[];
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  ruleState: EvaluatedRuleState;
  schema: PublishedSchemaResponse;
};

export function MetadataDrivenCreateForm({
  activeForm,
  appLogicalName,
  canCreate,
  entityDisplayName,
  fieldMap,
  formValues,
  isSaving,
  optionSets,
  onFieldValueChange,
  onSubmit,
  ruleState,
  schema,
}: MetadataDrivenCreateFormProps) {
  if (!activeForm || activeForm.tabs.length === 0) {
    return (
      <FlatCreateForm
        appLogicalName={appLogicalName}
        canCreate={canCreate}
        entityDisplayName={entityDisplayName}
        fields={schema.fields}
        formValues={formValues}
        isSaving={isSaving}
        optionSets={optionSets}
        onFieldValueChange={onFieldValueChange}
        onSubmit={onSubmit}
        ruleState={ruleState}
      />
    );
  }

  const visibleTabs = activeForm.tabs.filter((tab) => tab.visible);
  const showTabHeaders = visibleTabs.length > 1;

  return (
    <form
      className="space-y-4 rounded-md border border-emerald-100 bg-white p-4"
      onSubmit={onSubmit}
    >
      <div>
        <p className="text-sm font-medium text-zinc-800">
          New {entityDisplayName} Record
        </p>
        <p className="text-xs text-zinc-500">
          Using form: {activeForm.display_name} ({activeForm.form_type}) in {appLogicalName}.
        </p>
      </div>

      {visibleTabs.map((tab) => (
        <FormTabRenderer
          key={tab.logical_name}
          appLogicalName={appLogicalName}
          tab={tab}
          fieldMap={fieldMap}
          formValues={formValues}
          onFieldValueChange={onFieldValueChange}
          optionSets={optionSets}
          ruleState={ruleState}
          showTabHeader={showTabHeaders}
        />
      ))}

      <Button disabled={!canCreate || isSaving} type="submit">
        {isSaving ? "Saving..." : "Create Record"}
      </Button>
    </form>
  );
}

type FormTabRendererProps = {
  appLogicalName: string;
  tab: FormTab;
  fieldMap: Map<string, FieldResponse>;
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  optionSets: OptionSetResponse[];
  ruleState: EvaluatedRuleState;
  showTabHeader: boolean;
};

function FormTabRenderer({
  appLogicalName,
  tab,
  fieldMap,
  formValues,
  onFieldValueChange,
  optionSets,
  ruleState,
  showTabHeader,
}: FormTabRendererProps) {
  const visibleSections = tab.sections
    .filter((section) => section.visible)
    .sort((left, right) => left.position - right.position);

  return (
    <div className="space-y-4">
      {showTabHeader ? (
        <div className="border-b border-zinc-200 pb-1">
          <p className="text-sm font-semibold text-zinc-700">{tab.display_name}</p>
        </div>
      ) : null}

      {visibleSections.map((section) => (
        <FormSectionRenderer
          key={section.logical_name}
          appLogicalName={appLogicalName}
          section={section}
          fieldMap={fieldMap}
          formValues={formValues}
          onFieldValueChange={onFieldValueChange}
          optionSets={optionSets}
          ruleState={ruleState}
        />
      ))}
    </div>
  );
}

type FormSectionRendererProps = {
  appLogicalName: string;
  section: FormSection;
  fieldMap: Map<string, FieldResponse>;
  formValues: Record<string, unknown>;
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  optionSets: OptionSetResponse[];
  ruleState: EvaluatedRuleState;
};

function FormSectionRenderer({
  appLogicalName,
  section,
  fieldMap,
  formValues,
  onFieldValueChange,
  optionSets,
  ruleState,
}: FormSectionRendererProps) {
  const visibleFields = section.fields
    .filter((fieldPlacement) => {
      return fieldPlacement.visible && !ruleState.hiddenFieldNames.has(fieldPlacement.field_logical_name);
    })
    .sort((left, right) => left.position - right.position);

  if (visibleFields.length === 0) {
    return null;
  }

  const columnGroups: FormFieldPlacement[][] = [];
  for (let columnIndex = 0; columnIndex < section.columns; columnIndex += 1) {
    columnGroups.push(
      visibleFields
        .filter((fieldPlacement) => fieldPlacement.column === columnIndex)
        .sort((left, right) => left.position - right.position),
    );
  }

  const gridClass =
    section.columns === 3
      ? "grid gap-4 md:grid-cols-3"
      : section.columns === 2
        ? "grid gap-4 md:grid-cols-2"
        : "grid gap-4 grid-cols-1";

  return (
    <fieldset className="space-y-3">
      <legend className="text-xs font-semibold uppercase tracking-[0.12em] text-zinc-500">
        {section.display_name}
      </legend>
      <div className={gridClass}>
        {columnGroups.map((columnFields, columnIndex) => (
          <div key={`col-${String(columnIndex)}`} className="space-y-4">
            {columnFields.map((fieldPlacement) => (
              <FieldControl
                key={fieldPlacement.field_logical_name}
                appLogicalName={appLogicalName}
                placement={fieldPlacement}
                field={fieldMap.get(fieldPlacement.field_logical_name) ?? null}
                formValues={formValues}
                onFieldValueChange={onFieldValueChange}
                optionSets={optionSets}
                ruleState={ruleState}
              />
            ))}
          </div>
        ))}
      </div>

      {section.subgrids.length > 0 ? (
        <Notice tone="warning">
          This section includes {section.subgrids.length} sub-grid control(s). Sub-grids render on saved record detail pages.
        </Notice>
      ) : null}
    </fieldset>
  );
}

type FlatCreateFormProps = {
  appLogicalName: string;
  canCreate: boolean;
  entityDisplayName: string;
  fields: FieldResponse[];
  formValues: Record<string, unknown>;
  isSaving: boolean;
  optionSets: OptionSetResponse[];
  onFieldValueChange: (fieldLogicalName: string, value: unknown) => void;
  onSubmit: (event: FormEvent<HTMLFormElement>) => void;
  ruleState: EvaluatedRuleState;
};

function FlatCreateForm({
  appLogicalName,
  canCreate,
  entityDisplayName,
  fields,
  formValues,
  isSaving,
  optionSets,
  onFieldValueChange,
  onSubmit,
  ruleState,
}: FlatCreateFormProps) {
  return (
    <form
      className="space-y-4 rounded-md border border-emerald-100 bg-white p-4"
      onSubmit={onSubmit}
    >
      <div>
        <p className="text-sm font-medium text-zinc-800">New {entityDisplayName} Record</p>
        <p className="text-xs text-zinc-500">
          Fill fields and create a runtime row in {appLogicalName}.
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {fields
          .filter((field) => !ruleState.hiddenFieldNames.has(field.logical_name))
          .map((field) => (
            <FieldControl
              key={field.logical_name}
              appLogicalName={appLogicalName}
              placement={{
                field_logical_name: field.logical_name,
                column: 0,
                position: 0,
                visible: true,
                read_only: false,
                required_override: null,
                label_override: null,
              }}
              field={field}
              formValues={formValues}
              onFieldValueChange={onFieldValueChange}
              optionSets={optionSets}
              ruleState={ruleState}
            />
          ))}
      </div>

      <Button disabled={!canCreate || isSaving} type="submit">
        {isSaving ? "Saving..." : "Create Record"}
      </Button>
    </form>
  );
}
