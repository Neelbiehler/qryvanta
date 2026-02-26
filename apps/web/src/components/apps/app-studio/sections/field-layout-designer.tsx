import { type DragEvent, useMemo, useState } from "react";

import { Button, Input, Label, Notice, Select } from "@qryvanta/ui";

import type { FieldResponse } from "@/lib/api";
import { FieldDropZone } from "@/components/apps/app-studio/sections/field-drop-zone";
import { SurfaceCatalogEditor } from "@/components/apps/app-studio/sections/surface-catalog-editor";
import type { AppSurfaceDraft } from "@/components/apps/app-studio/sections/types";
import {
  appendUniqueField,
  buildNextLogicalName,
  moveField,
  normalizeSurfaceLogicalName,
  parseFieldLogicalNames,
  serializeFieldLogicalNames,
} from "@/components/apps/app-studio/sections/field-layout-utils";

type FieldLayoutDesignerProps = {
  selectedEntityFields: FieldResponse[];
  forms: AppSurfaceDraft[];
  listViews: AppSurfaceDraft[];
  defaultFormLogicalName: string;
  defaultListViewLogicalName: string;
  onChangeForms: (value: AppSurfaceDraft[]) => void;
  onChangeListViews: (value: AppSurfaceDraft[]) => void;
  onChangeDefaultFormLogicalName: (value: string) => void;
  onChangeDefaultListViewLogicalName: (value: string) => void;
};

export function FieldLayoutDesigner({
  selectedEntityFields,
  forms,
  listViews,
  defaultFormLogicalName,
  defaultListViewLogicalName,
  onChangeForms,
  onChangeListViews,
  onChangeDefaultFormLogicalName,
  onChangeDefaultListViewLogicalName,
}: FieldLayoutDesignerProps) {
  const [activeDropLineId, setActiveDropLineId] = useState<string | null>(null);
  const [dragLabel, setDragLabel] = useState<string | null>(null);

  const selectedForm = useMemo(
    () =>
      forms.find((form) => form.logicalName === defaultFormLogicalName) ??
      forms.at(0) ?? {
        logicalName: "main_form",
        displayName: "Main Form",
        fieldLogicalNames: [],
      },
    [defaultFormLogicalName, forms],
  );

  const selectedListView = useMemo(
    () =>
      listViews.find((view) => view.logicalName === defaultListViewLogicalName) ??
      listViews.at(0) ?? {
        logicalName: "main_view",
        displayName: "Main View",
        fieldLogicalNames: [],
      },
    [defaultListViewLogicalName, listViews],
  );

  const fieldLabelByLogicalName = useMemo(() => {
    const dictionary = new Map<string, string>();
    for (const field of selectedEntityFields) {
      dictionary.set(field.logical_name, `${field.display_name} (${field.logical_name})`);
    }
    return dictionary;
  }, [selectedEntityFields]);

  const availableFields = useMemo(() => {
    const included = new Set([
      ...selectedForm.fieldLogicalNames,
      ...selectedListView.fieldLogicalNames,
    ]);
    return selectedEntityFields.filter((field) => !included.has(field.logical_name));
  }, [selectedEntityFields, selectedForm.fieldLogicalNames, selectedListView.fieldLogicalNames]);

  function updateForm(next: AppSurfaceDraft) {
    onChangeForms(
      forms.map((form) => (form.logicalName === selectedForm.logicalName ? next : form)),
    );
  }

  function updateListView(next: AppSurfaceDraft) {
    onChangeListViews(
      listViews.map((view) =>
        view.logicalName === selectedListView.logicalName ? next : view,
      ),
    );
  }

  function onDragStart(
    event: DragEvent<HTMLButtonElement>,
    logicalName: string,
    source: "available" | "form" | "view",
  ) {
    event.dataTransfer.setData("text/app-field-logical-name", logicalName);
    event.dataTransfer.setData("text/app-field-source", source);
    event.dataTransfer.effectAllowed = "move";
    const field = selectedEntityFields.find((candidate) => candidate.logical_name === logicalName);
    setDragLabel(field?.display_name ?? logicalName);
  }

  function onDropToTargetAtIndex(
    event: DragEvent<HTMLDivElement>,
    target: "form" | "view",
    index: number | null,
  ) {
    event.preventDefault();
    setDragLabel(null);

    const logicalName = event.dataTransfer.getData("text/app-field-logical-name").trim();
    const source = (event.dataTransfer.getData("text/app-field-source") || "available") as
      | "available"
      | "form"
      | "view";
    if (!logicalName) {
      return;
    }

    const sourceFields =
      source === "form"
        ? selectedForm.fieldLogicalNames
        : source === "view"
          ? selectedListView.fieldLogicalNames
          : [];
    const targetFields =
      target === "form" ? selectedForm.fieldLogicalNames : selectedListView.fieldLogicalNames;

    const normalizedTarget = targetFields.filter((field) => field !== logicalName);
    const targetIndex =
      index === null ? normalizedTarget.length : Math.max(0, Math.min(index, normalizedTarget.length));
    const insertedTarget = [...normalizedTarget];
    insertedTarget.splice(targetIndex, 0, logicalName);

    const normalizedSource =
      source === "available" || source === target
        ? sourceFields
        : sourceFields.filter((field) => field !== logicalName);

    if (target === "form") {
      if (source === "view") {
        updateListView({ ...selectedListView, fieldLogicalNames: normalizedSource });
      }
      updateForm({ ...selectedForm, fieldLogicalNames: insertedTarget });
    } else {
      if (source === "form") {
        updateForm({ ...selectedForm, fieldLogicalNames: normalizedSource });
      }
      updateListView({ ...selectedListView, fieldLogicalNames: insertedTarget });
    }
  }

  function createSurface(base: "form" | "view") {
    const source = base === "form" ? forms : listViews;
    const nextIndex = source.length + 1;
    const logicalPrefix = base === "form" ? "form" : "view";
    const displayPrefix = base === "form" ? "Form" : "View";
    const logicalName = buildNextLogicalName(
      source.map((surface) => surface.logicalName),
      logicalPrefix,
      nextIndex,
    );
    const nextSurface: AppSurfaceDraft = {
      logicalName,
      displayName: `${displayPrefix} ${nextIndex}`,
      fieldLogicalNames: [],
    };

    if (base === "form") {
      onChangeForms([...forms, nextSurface]);
      onChangeDefaultFormLogicalName(logicalName);
      return;
    }

    onChangeListViews([...listViews, nextSurface]);
    onChangeDefaultListViewLogicalName(logicalName);
  }

  function renameForm(formLogicalName: string, key: "logicalName" | "displayName", value: string) {
    const nextValue = key === "logicalName" ? normalizeSurfaceLogicalName(value) : value;
    const nextForms = forms.map((form) =>
      form.logicalName === formLogicalName ? { ...form, [key]: nextValue } : form,
    );
    onChangeForms(nextForms);
    if (key === "logicalName" && defaultFormLogicalName === formLogicalName) {
      onChangeDefaultFormLogicalName(nextValue);
    }
  }

  function renameListView(
    viewLogicalName: string,
    key: "logicalName" | "displayName",
    value: string,
  ) {
    const nextValue = key === "logicalName" ? normalizeSurfaceLogicalName(value) : value;
    const nextViews = listViews.map((view) =>
      view.logicalName === viewLogicalName ? { ...view, [key]: nextValue } : view,
    );
    onChangeListViews(nextViews);
    if (key === "logicalName" && defaultListViewLogicalName === viewLogicalName) {
      onChangeDefaultListViewLogicalName(nextValue);
    }
  }

  function deleteForm(formLogicalName: string) {
    if (forms.length <= 1 || formLogicalName === defaultFormLogicalName) {
      return;
    }
    onChangeForms(forms.filter((form) => form.logicalName !== formLogicalName));
  }

  function deleteListView(viewLogicalName: string) {
    if (listViews.length <= 1 || viewLogicalName === defaultListViewLogicalName) {
      return;
    }
    onChangeListViews(listViews.filter((view) => view.logicalName !== viewLogicalName));
  }

  return (
    <div className="space-y-3 rounded-md border border-zinc-200 bg-zinc-50 p-3">
      <div>
        <p className="text-sm font-semibold text-zinc-900">Form and View Designer</p>
        <p className="text-xs text-zinc-600">
          Build multiple forms and list views. The selected defaults are used as worker runtime surfaces.
        </p>
      </div>

      <SurfaceDefaults
        forms={forms}
        listViews={listViews}
        selectedFormLogicalName={selectedForm.logicalName}
        selectedListViewLogicalName={selectedListView.logicalName}
        onAddForm={() => createSurface("form")}
        onAddView={() => createSurface("view")}
        onChangeDefaultFormLogicalName={onChangeDefaultFormLogicalName}
        onChangeDefaultListViewLogicalName={onChangeDefaultListViewLogicalName}
      />

      <div className="grid gap-3 md:grid-cols-2">
        <SurfaceCatalogEditor
          title="Form Catalog"
          surfaces={forms}
          defaultLogicalName={defaultFormLogicalName}
          onSetDefault={onChangeDefaultFormLogicalName}
          onRename={renameForm}
          onDelete={deleteForm}
        />

        <SurfaceCatalogEditor
          title="View Catalog"
          surfaces={listViews}
          defaultLogicalName={defaultListViewLogicalName}
          onSetDefault={onChangeDefaultListViewLogicalName}
          onRename={renameListView}
          onDelete={deleteListView}
        />
      </div>

      <p className="text-[11px] text-zinc-500">
        Keep at least one form and one view. Switch default before deleting an active default.
      </p>

      <FieldLayoutBoard
        availableFields={availableFields}
        fieldLabelByLogicalName={fieldLabelByLogicalName}
        selectedForm={selectedForm}
        selectedListView={selectedListView}
        activeDropLineId={activeDropLineId}
        onSetActiveDropLineId={setActiveDropLineId}
        onDragStart={onDragStart}
        onDragEnd={() => setDragLabel(null)}
        onUpdateForm={updateForm}
        onUpdateListView={updateListView}
        onDropToTargetAtIndex={onDropToTargetAtIndex}
      />

      {dragLabel ? (
        <Notice tone="neutral">Dragging `{dragLabel}` - drop on highlighted insertion line.</Notice>
      ) : null}

      <AdvancedTextMapping
        selectedForm={selectedForm}
        selectedListView={selectedListView}
        onUpdateForm={updateForm}
        onUpdateListView={updateListView}
      />
    </div>
  );
}

type SurfaceDefaultsProps = {
  forms: AppSurfaceDraft[];
  listViews: AppSurfaceDraft[];
  selectedFormLogicalName: string;
  selectedListViewLogicalName: string;
  onAddForm: () => void;
  onAddView: () => void;
  onChangeDefaultFormLogicalName: (value: string) => void;
  onChangeDefaultListViewLogicalName: (value: string) => void;
};

function SurfaceDefaults({
  forms,
  listViews,
  selectedFormLogicalName,
  selectedListViewLogicalName,
  onAddForm,
  onAddView,
  onChangeDefaultFormLogicalName,
  onChangeDefaultListViewLogicalName,
}: SurfaceDefaultsProps) {
  return (
    <div className="grid gap-3 md:grid-cols-2">
      <div className="space-y-2 rounded-md border border-zinc-200 bg-white p-2">
        <div className="flex items-center justify-between gap-2">
          <Label htmlFor="bind_default_form">Default Form</Label>
          <Button type="button" size="sm" variant="outline" onClick={onAddForm}>
            Add Form
          </Button>
        </div>
        <Select
          id="bind_default_form"
          value={selectedFormLogicalName}
          onChange={(event) => onChangeDefaultFormLogicalName(event.target.value)}
        >
          {forms.map((form) => (
            <option key={`default-form-${form.logicalName}`} value={form.logicalName}>
              {form.displayName} ({form.logicalName})
            </option>
          ))}
        </Select>
      </div>

      <div className="space-y-2 rounded-md border border-zinc-200 bg-white p-2">
        <div className="flex items-center justify-between gap-2">
          <Label htmlFor="bind_default_list_view">Default List View</Label>
          <Button type="button" size="sm" variant="outline" onClick={onAddView}>
            Add View
          </Button>
        </div>
        <Select
          id="bind_default_list_view"
          value={selectedListViewLogicalName}
          onChange={(event) => onChangeDefaultListViewLogicalName(event.target.value)}
        >
          {listViews.map((view) => (
            <option key={`default-view-${view.logicalName}`} value={view.logicalName}>
              {view.displayName} ({view.logicalName})
            </option>
          ))}
        </Select>
      </div>
    </div>
  );
}

type FieldLayoutBoardProps = {
  availableFields: FieldResponse[];
  fieldLabelByLogicalName: Map<string, string>;
  selectedForm: AppSurfaceDraft;
  selectedListView: AppSurfaceDraft;
  activeDropLineId: string | null;
  onSetActiveDropLineId: (value: string | null) => void;
  onDragStart: (
    event: DragEvent<HTMLButtonElement>,
    logicalName: string,
    source: "available" | "form" | "view",
  ) => void;
  onDragEnd: () => void;
  onUpdateForm: (next: AppSurfaceDraft) => void;
  onUpdateListView: (next: AppSurfaceDraft) => void;
  onDropToTargetAtIndex: (
    event: DragEvent<HTMLDivElement>,
    target: "form" | "view",
    index: number | null,
  ) => void;
};

function FieldLayoutBoard({
  availableFields,
  fieldLabelByLogicalName,
  selectedForm,
  selectedListView,
  activeDropLineId,
  onSetActiveDropLineId,
  onDragStart,
  onDragEnd,
  onUpdateForm,
  onUpdateListView,
  onDropToTargetAtIndex,
}: FieldLayoutBoardProps) {
  return (
    <div className="grid gap-3 lg:grid-cols-[1fr_1fr_1fr]">
      <div className="space-y-2 rounded-md border border-zinc-200 bg-white p-2">
        <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500">
          Available Fields
        </p>
        <div className="max-h-48 space-y-1 overflow-y-auto pr-1">
          {availableFields.length > 0 ? (
            availableFields.map((field) => (
              <button
                key={`available-${field.logical_name}`}
                type="button"
                draggable
                onDragStart={(event) => onDragStart(event, field.logical_name, "available")}
                className="w-full rounded-md border border-zinc-200 px-2 py-1 text-left text-xs transition hover:border-emerald-300"
              >
                <p className="font-medium text-zinc-900">{field.display_name}</p>
                <p className="font-mono text-[10px] text-zinc-500">{field.logical_name}</p>
              </button>
            ))
          ) : (
            <p className="text-[11px] text-zinc-500">No remaining fields.</p>
          )}
        </div>
      </div>

      <FieldDropZone
        title={selectedForm.displayName}
        helperText="Create/edit surface"
        dragSource="form"
        fieldLogicalNames={selectedForm.fieldLogicalNames}
        fieldLabelByLogicalName={fieldLabelByLogicalName}
        onAddField={(logicalName) =>
          onUpdateForm({
            ...selectedForm,
            fieldLogicalNames: appendUniqueField(selectedForm.fieldLogicalNames, logicalName),
          })
        }
        onMoveField={(logicalName, direction) =>
          onUpdateForm({
            ...selectedForm,
            fieldLogicalNames: moveField(selectedForm.fieldLogicalNames, logicalName, direction),
          })
        }
        onRemoveField={(logicalName) =>
          onUpdateForm({
            ...selectedForm,
            fieldLogicalNames: selectedForm.fieldLogicalNames.filter(
              (field) => field !== logicalName,
            ),
          })
        }
        onDrop={(event) => onDropToTargetAtIndex(event, "form", null)}
        onDropAtIndex={(event, index) => onDropToTargetAtIndex(event, "form", index)}
        dropLinePrefix={`form-${selectedForm.logicalName}`}
        activeDropLineId={activeDropLineId}
        onSetActiveDropLineId={onSetActiveDropLineId}
        onDragStart={onDragStart}
        onDragEnd={onDragEnd}
      />

      <FieldDropZone
        title={selectedListView.displayName}
        helperText="Grid/list columns"
        dragSource="view"
        fieldLogicalNames={selectedListView.fieldLogicalNames}
        fieldLabelByLogicalName={fieldLabelByLogicalName}
        onAddField={(logicalName) =>
          onUpdateListView({
            ...selectedListView,
            fieldLogicalNames: appendUniqueField(selectedListView.fieldLogicalNames, logicalName),
          })
        }
        onMoveField={(logicalName, direction) =>
          onUpdateListView({
            ...selectedListView,
            fieldLogicalNames: moveField(selectedListView.fieldLogicalNames, logicalName, direction),
          })
        }
        onRemoveField={(logicalName) =>
          onUpdateListView({
            ...selectedListView,
            fieldLogicalNames: selectedListView.fieldLogicalNames.filter(
              (field) => field !== logicalName,
            ),
          })
        }
        onDrop={(event) => onDropToTargetAtIndex(event, "view", null)}
        onDropAtIndex={(event, index) => onDropToTargetAtIndex(event, "view", index)}
        dropLinePrefix={`view-${selectedListView.logicalName}`}
        activeDropLineId={activeDropLineId}
        onSetActiveDropLineId={onSetActiveDropLineId}
        onDragStart={onDragStart}
        onDragEnd={onDragEnd}
      />
    </div>
  );
}

type AdvancedTextMappingProps = {
  selectedForm: AppSurfaceDraft;
  selectedListView: AppSurfaceDraft;
  onUpdateForm: (next: AppSurfaceDraft) => void;
  onUpdateListView: (next: AppSurfaceDraft) => void;
};

function AdvancedTextMapping({
  selectedForm,
  selectedListView,
  onUpdateForm,
  onUpdateListView,
}: AdvancedTextMappingProps) {
  return (
    <details className="rounded-md border border-zinc-200 bg-white p-2">
      <summary className="cursor-pointer text-xs font-semibold uppercase tracking-wide text-zinc-500">
        Advanced text mapping
      </summary>
      <div className="mt-2 grid gap-2 md:grid-cols-2">
        <div className="space-y-1">
          <Label htmlFor="bind_form_fields">Form fields</Label>
          <Input
            id="bind_form_fields"
            value={serializeFieldLogicalNames(selectedForm.fieldLogicalNames)}
            onChange={(event) =>
              onUpdateForm({
                ...selectedForm,
                fieldLogicalNames: parseFieldLogicalNames(event.target.value),
              })
            }
            placeholder="name, email, owner"
          />
        </div>
        <div className="space-y-1">
          <Label htmlFor="bind_list_fields">List fields</Label>
          <Input
            id="bind_list_fields"
            value={serializeFieldLogicalNames(selectedListView.fieldLogicalNames)}
            onChange={(event) =>
              onUpdateListView({
                ...selectedListView,
                fieldLogicalNames: parseFieldLogicalNames(event.target.value),
              })
            }
            placeholder="name, status, updated_at"
          />
        </div>
      </div>
    </details>
  );
}
