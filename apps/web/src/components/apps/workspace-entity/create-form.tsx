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

const AUTO_GENERATED_FIELD_NAMES = new Set([
  "record_id",
  "subject_record_id",
  "subject",
]);

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
      className="space-y-5 rounded-xl border border-emerald-100 bg-white p-5 shadow-sm"
      onSubmit={onSubmit}
    >
      <div className="border-b border-emerald-50 pb-3">
        <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-700">
          New Record
        </p>
        <p className="mt-0.5 text-base font-semibold text-zinc-900">
          {entityDisplayName}
        </p>
        <p className="text-xs text-zinc-400">
          {activeForm.display_name} &middot; {activeForm.form_type} &middot; {appLogicalName}
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
        {isSaving ? "Saving…" : "Create Record"}
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
        <div className="border-b border-emerald-100 pb-1.5">
          <p className="text-xs font-semibold uppercase tracking-[0.12em] text-emerald-700">
            {tab.display_name}
          </p>
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
      return (
        fieldPlacement.visible &&
        !AUTO_GENERATED_FIELD_NAMES.has(fieldPlacement.field_logical_name) &&
        !ruleState.hiddenFieldNames.has(fieldPlacement.field_logical_name)
      );
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
    <fieldset className="space-y-3 rounded-lg border border-emerald-50 bg-emerald-50/30 p-3">
      <legend className="-ml-1 px-1 text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-700">
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
      className="space-y-5 rounded-xl border border-emerald-100 bg-white p-5 shadow-sm"
      onSubmit={onSubmit}
    >
      <div className="border-b border-emerald-50 pb-3">
        <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-emerald-700">
          New Record
        </p>
        <p className="mt-0.5 text-base font-semibold text-zinc-900">{entityDisplayName}</p>
        <p className="text-xs text-zinc-400">{appLogicalName}</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {fields
          .filter(
            (field) =>
              !AUTO_GENERATED_FIELD_NAMES.has(field.logical_name) &&
              !ruleState.hiddenFieldNames.has(field.logical_name),
          )
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
        {isSaving ? "Saving…" : "Create Record"}
      </Button>
    </form>
  );
}
