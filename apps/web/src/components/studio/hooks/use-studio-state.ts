"use client";

import { useCallback, useMemo, useState } from "react";

import {
  apiFetch,
  type AppEntityBindingResponse,
  type BindAppEntityRequest,
  type AppResponse,
  type CreateFieldRequest,
  type CreateFormRequest,
  type CreateViewRequest,
  type EntityResponse,
  type FieldResponse,
  type FormResponse,
  type PublishedSchemaResponse,
  type RoleResponse,
  type RuntimeRecordResponse,
  type ViewResponse,
} from "@/lib/api";
import type { EntityTreeNode, StudioSelection } from "@/components/studio/types";
import {
  type FormEditorState,
  normalizeHeaderFields,
  normalizeTabs,
  useFormEditorState,
} from "@/components/studio/hooks/use-form-editor-state";
import {
  type ViewEditorState,
  useViewEditorState,
} from "@/components/studio/hooks/use-view-editor-state";

function collectFormFieldNames(tabs: unknown[] | undefined): string[] {
  const names = new Set<string>();
  for (const tab of normalizeTabs(tabs)) {
    for (const section of tab.sections) {
      for (const placement of section.fields) {
        names.add(placement.field_logical_name);
      }
    }
  }
  return [...names];
}

function collectViewFieldNames(columns: unknown[] | undefined): string[] {
  if (!Array.isArray(columns)) return [];
  const names = new Set<string>();
  for (const candidate of columns) {
    if (!candidate || typeof candidate !== "object") continue;
    const column = candidate as { field_logical_name?: unknown };
    if (typeof column.field_logical_name !== "string") continue;
    names.add(column.field_logical_name);
  }
  return [...names];
}

// ---------------------------------------------------------------------------
// Public controller type
// ---------------------------------------------------------------------------

export type StudioController = {
  // App context
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
  selectedApp: string;
  setSelectedApp: (appLogicalName: string) => void;
  selectedAppDisplayName: string;

  // Navigation
  selection: StudioSelection;
  setSelection: (selection: StudioSelection) => void;

  // Entity tree data (lazily loaded)
  entityTree: EntityTreeNode[];
  isLoadingEntity: boolean;
  expandedEntities: Set<string>;
  toggleEntityExpanded: (entityLogicalName: string) => void;
  createEntity: (input: { logicalName: string; displayName: string }) => Promise<boolean>;
  createField: (
    entityLogicalName: string,
    input: {
      logicalName: string;
      displayName: string;
      fieldType: CreateFieldRequest["field_type"];
      isRequired: boolean;
      relationTargetEntity: string;
    },
  ) => Promise<boolean>;
  createForm: (
    entityLogicalName: string,
    input: {
      logicalName: string;
      displayName: string;
      formType: "main" | "quick_create" | "quick_view";
    },
  ) => Promise<string | null>;
  createView: (
    entityLogicalName: string,
    input: {
      logicalName: string;
      displayName: string;
      viewType: string;
    },
  ) => Promise<string | null>;

  // Cached data accessors
  getEntityFields: (entityLogicalName: string) => FieldResponse[];
  getPublishedSchema: (entityLogicalName: string) => PublishedSchemaResponse | null;
  refreshEntityPreviewRecords: (entityLogicalName: string) => Promise<void>;

  // Form editor (active when selection.kind === "form")
  formEditor: FormEditorState | null;
  formMeta: { logicalName: string; displayName: string; formType: string; headerFieldsText: string } | null;
  setFormMeta: (patch: Partial<{ displayName: string; formType: string; headerFieldsText: string }>) => void;

  // View editor (active when selection.kind === "view")
  viewEditor: ViewEditorState | null;
  viewMeta: { logicalName: string; displayName: string; viewType: string; isDefault: boolean } | null;
  setViewMeta: (patch: Partial<{ displayName: string; viewType: string; isDefault: boolean }>) => void;

  // Save / status
  isSaving: boolean;
  errorMessage: string | null;
  statusMessage: string | null;
  clearMessages: () => void;
  handleSaveForm: () => Promise<void>;
  handleSaveView: () => Promise<void>;
};

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

type UseStudioStateInput = {
  initialAppLogicalName: string;
  apps: AppResponse[];
  entities: EntityResponse[];
  roles: RoleResponse[];
  bindings: AppEntityBindingResponse[];
};

export function useStudioState({
  initialAppLogicalName,
  apps,
  entities,
  roles,
  bindings,
}: UseStudioStateInput): StudioController {
  // ---- Top-level selection ----
  const [selectedApp, setSelectedApp] = useState(
    apps.some((app) => app.logical_name === initialAppLogicalName)
      ? initialAppLogicalName
      : (apps.at(0)?.logical_name ?? ""),
  );
  const [selection, setSelectionRaw] = useState<StudioSelection>({ kind: "overview" });
  const [entitiesState, setEntitiesState] = useState<EntityResponse[]>(entities);

  // ---- Entity tree ----
  const [expandedEntities, setExpandedEntities] = useState<Set<string>>(new Set());
  const [isLoadingEntity, setIsLoadingEntity] = useState(false);

  // ---- Entity data caches ----
  const [formsCache, setFormsCache] = useState<Map<string, FormResponse[]>>(new Map());
  const [viewsCache, setViewsCache] = useState<Map<string, ViewResponse[]>>(new Map());
  const [fieldsCache, setFieldsCache] = useState<Map<string, FieldResponse[]>>(new Map());
  const [publishedSchemaCache, setPublishedSchemaCache] = useState<
    Map<string, PublishedSchemaResponse>
  >(new Map());
  const [previewRecordsCache, setPreviewRecordsCache] = useState<
    Map<string, RuntimeRecordResponse[]>
  >(new Map());
  const [bindingsCacheByApp, setBindingsCacheByApp] = useState<
    Map<string, AppEntityBindingResponse[]>
  >(() => new Map([[initialAppLogicalName, bindings]]));

  // ---- Form editor metadata ----
  const [activeFormResponse, setActiveFormResponse] = useState<FormResponse | null>(null);
  const [formDisplayName, setFormDisplayName] = useState("");
  const [formType, setFormType] = useState("main");
  const [headerFieldsText, setHeaderFieldsText] = useState("");
  const [activeViewResponse, setActiveViewResponse] = useState<ViewResponse | null>(null);
  const [viewDisplayName, setViewDisplayName] = useState("");
  const [viewType, setViewType] = useState("grid");
  const [viewIsDefault, setViewIsDefault] = useState(false);

  // ---- Status ----
  const [isSaving, setIsSaving] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  // ---- Form editor hook (only active when editing a form) ----
  const formEditor = useFormEditorState({
    initialTabs: activeFormResponse?.tabs,
    onError: setErrorMessage,
  });
  const viewEditor = useViewEditorState({
    initialColumns: activeViewResponse?.columns,
    initialDefaultSort: activeViewResponse?.default_sort,
    initialFilterCriteria: activeViewResponse?.filter_criteria,
    previewRecords:
      selection.kind === "view"
        ? (previewRecordsCache.get(selection.entityLogicalName) ?? [])
        : [],
    publishedFields:
      selection.kind === "view"
        ? (publishedSchemaCache.get(selection.entityLogicalName)?.fields ?? [])
        : [],
  });

  // ---- Entity data loading ----

  async function loadEntityData(entityLogicalName: string): Promise<void> {
    if (formsCache.has(entityLogicalName)) return;

    setIsLoadingEntity(true);
    try {
      const [formsRes, viewsRes, fieldsRes, schemaRes] = await Promise.all([
        apiFetch(`/api/entities/${encodeURIComponent(entityLogicalName)}/forms`),
        apiFetch(`/api/entities/${encodeURIComponent(entityLogicalName)}/views`),
        apiFetch(`/api/entities/${encodeURIComponent(entityLogicalName)}/fields`),
        apiFetch(`/api/entities/${encodeURIComponent(entityLogicalName)}/published`),
      ]);

      if (formsRes.ok) {
        const forms = (await formsRes.json()) as FormResponse[];
        setFormsCache((prev) => new Map(prev).set(entityLogicalName, forms));
      }
      if (viewsRes.ok) {
        const views = (await viewsRes.json()) as ViewResponse[];
        setViewsCache((prev) => new Map(prev).set(entityLogicalName, views));
      }
      if (fieldsRes.ok) {
        const fields = (await fieldsRes.json()) as FieldResponse[];
        setFieldsCache((prev) => new Map(prev).set(entityLogicalName, fields));
      }
      if (schemaRes.ok) {
        const schema = (await schemaRes.json()) as PublishedSchemaResponse;
        setPublishedSchemaCache((prev) => new Map(prev).set(entityLogicalName, schema));
      }

      const previewRes = await apiFetch(
        `/api/runtime/${encodeURIComponent(entityLogicalName)}/records?limit=50&offset=0`,
      );
      if (previewRes.ok) {
        const records = (await previewRes.json()) as RuntimeRecordResponse[];
        setPreviewRecordsCache((prev) => new Map(prev).set(entityLogicalName, records));
      }
    } catch {
      setErrorMessage(`Failed to load data for entity "${entityLogicalName}".`);
    } finally {
      setIsLoadingEntity(false);
    }
  }

  const refreshEntityPreviewRecords = useCallback(
    async (entityLogicalName: string): Promise<void> => {
      try {
        const previewRes = await apiFetch(
          `/api/runtime/${encodeURIComponent(entityLogicalName)}/records?limit=50&offset=0`,
        );
        if (!previewRes.ok) return;
        const records = (await previewRes.json()) as RuntimeRecordResponse[];
        setPreviewRecordsCache((prev) => new Map(prev).set(entityLogicalName, records));
      } catch {
        // best-effort refresh only
      }
    },
    [],
  );

  const syncAppBindingForEntity = useCallback(
    async (
      entityLogicalName: string,
      options: {
        preferredFormLogicalName?: string;
        preferredViewLogicalName?: string;
      } = {},
    ): Promise<boolean> => {
      if (!selectedApp) return false;

      try {
        let appBindings = bindingsCacheByApp.get(selectedApp) ?? [];
        if (appBindings.length === 0) {
          const bindingsRes = await apiFetch(
            `/api/apps/${encodeURIComponent(selectedApp)}/entities`,
          );
          if (bindingsRes.ok) {
            appBindings = (await bindingsRes.json()) as AppEntityBindingResponse[];
            setBindingsCacheByApp((prev) => new Map(prev).set(selectedApp, appBindings));
          }
        }

        let forms = formsCache.get(entityLogicalName) ?? [];
        if (forms.length === 0) {
          const formsRes = await apiFetch(
            `/api/entities/${encodeURIComponent(entityLogicalName)}/forms`,
          );
          if (!formsRes.ok) return false;
          forms = (await formsRes.json()) as FormResponse[];
          setFormsCache((prev) => new Map(prev).set(entityLogicalName, forms));
        }

        let views = viewsCache.get(entityLogicalName) ?? [];
        if (views.length === 0) {
          const viewsRes = await apiFetch(
            `/api/entities/${encodeURIComponent(entityLogicalName)}/views`,
          );
          if (!viewsRes.ok) return false;
          views = (await viewsRes.json()) as ViewResponse[];
          setViewsCache((prev) => new Map(prev).set(entityLogicalName, views));
        }

        if (forms.length === 0 || views.length === 0) return false;

        const appForms = forms.map((form) => ({
          logical_name: form.logical_name,
          display_name: form.display_name,
          field_logical_names: collectFormFieldNames(form.tabs),
        }));
        const appViews = views.map((view) => ({
          logical_name: view.logical_name,
          display_name: view.display_name,
          field_logical_names: collectViewFieldNames(view.columns),
        }));

        const existingBinding = appBindings.find(
          (binding) => binding.entity_logical_name === entityLogicalName,
        );

        const defaultFormLogicalName =
          (existingBinding &&
            appForms.some(
              (form) => form.logical_name === existingBinding.default_form_logical_name,
            ) &&
            existingBinding.default_form_logical_name) ||
          (options.preferredFormLogicalName &&
            appForms.some((form) => form.logical_name === options.preferredFormLogicalName) &&
            options.preferredFormLogicalName) ||
          appForms[0].logical_name;

        const defaultListViewLogicalName =
          (existingBinding &&
            appViews.some(
              (view) =>
                view.logical_name === existingBinding.default_list_view_logical_name,
            ) &&
            existingBinding.default_list_view_logical_name) ||
          (options.preferredViewLogicalName &&
            appViews.some((view) => view.logical_name === options.preferredViewLogicalName) &&
            options.preferredViewLogicalName) ||
          appViews[0].logical_name;

        const payload: BindAppEntityRequest = {
          entity_logical_name: entityLogicalName,
          navigation_label: existingBinding?.navigation_label ?? null,
          navigation_order:
            existingBinding?.navigation_order ??
            appBindings.reduce(
              (max, binding) => Math.max(max, binding.navigation_order),
              -1,
            ) + 1,
          forms: appForms,
          list_views: appViews,
          default_form_logical_name: defaultFormLogicalName,
          default_list_view_logical_name: defaultListViewLogicalName,
          form_field_logical_names:
            appForms.find((form) => form.logical_name === defaultFormLogicalName)
              ?.field_logical_names ?? [],
          list_field_logical_names:
            appViews.find((view) => view.logical_name === defaultListViewLogicalName)
              ?.field_logical_names ?? [],
          default_view_mode: existingBinding?.default_view_mode ?? "grid",
        };

        const response = await apiFetch(
          `/api/apps/${encodeURIComponent(selectedApp)}/entities`,
          {
            method: "POST",
            body: JSON.stringify(payload),
          },
        );
        if (!response.ok) {
          return false;
        }

        const bindingsRes = await apiFetch(
          `/api/apps/${encodeURIComponent(selectedApp)}/entities`,
        );
        if (bindingsRes.ok) {
          const nextBindings = (await bindingsRes.json()) as AppEntityBindingResponse[];
          setBindingsCacheByApp((prev) => new Map(prev).set(selectedApp, nextBindings));
        }

        return true;
      } catch {
        return false;
      }
    },
    [bindingsCacheByApp, formsCache, selectedApp, viewsCache],
  );

  const toggleEntityExpanded = useCallback(
    (entityLogicalName: string) => {
      setExpandedEntities((prev) => {
        const next = new Set(prev);
        if (next.has(entityLogicalName)) {
          next.delete(entityLogicalName);
        } else {
          next.add(entityLogicalName);
          void loadEntityData(entityLogicalName);
        }
        return next;
      });
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [formsCache],
  );

  // ---- Entity tree (built from caches) ----

  const entityTree: EntityTreeNode[] = useMemo(
    () =>
      entitiesState.map((entity) => ({
        logicalName: entity.logical_name,
        displayName: entity.display_name,
        icon: entity.icon ?? undefined,
        forms: (formsCache.get(entity.logical_name) ?? []).map((f) => ({
          logicalName: f.logical_name,
          displayName: f.display_name,
          formType: f.form_type,
        })),
        views: (viewsCache.get(entity.logical_name) ?? []).map((v) => ({
          logicalName: v.logical_name,
          displayName: v.display_name,
          viewType: v.view_type,
        })),
        businessRules: [],
      })),
    [entitiesState, formsCache, viewsCache],
  );

  async function createEntity(input: {
    logicalName: string;
    displayName: string;
  }): Promise<boolean> {
    const logicalName = input.logicalName.trim();
    const displayName = input.displayName.trim();
    if (!logicalName || !displayName) {
      setErrorMessage("Logical name and display name are required.");
      return false;
    }

    setIsSaving(true);
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const response = await apiFetch("/api/entities", {
        method: "POST",
        body: JSON.stringify({
          logical_name: logicalName,
          display_name: displayName,
        }),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to create entity.");
        return false;
      }

      const created = (await response.json()) as EntityResponse;
      setEntitiesState((current) => {
        if (current.some((entity) => entity.logical_name === created.logical_name)) {
          return current;
        }
        return [...current, created].sort((left, right) =>
          left.logical_name.localeCompare(right.logical_name),
        );
      });
      setExpandedEntities((current) => new Set(current).add(created.logical_name));
      await loadEntityData(created.logical_name);
      setStatusMessage(`Entity '${created.logical_name}' created.`);
      return true;
    } catch {
      setErrorMessage("Unable to create entity.");
      return false;
    } finally {
      setIsSaving(false);
    }
  }

  async function createField(
    entityLogicalName: string,
    input: {
      logicalName: string;
      displayName: string;
      fieldType: CreateFieldRequest["field_type"];
      isRequired: boolean;
      relationTargetEntity: string;
    },
  ): Promise<boolean> {
    const logicalName = input.logicalName.trim();
    const displayName = input.displayName.trim();
    if (!logicalName || !displayName) {
      setErrorMessage("Field logical name and display name are required.");
      return false;
    }
    if (input.fieldType === "relation" && !input.relationTargetEntity.trim()) {
      setErrorMessage("Relation fields require a target entity.");
      return false;
    }

    setIsSaving(true);
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const payload: CreateFieldRequest = {
        logical_name: logicalName,
        display_name: displayName,
        field_type: input.fieldType,
        is_required: input.isRequired,
        is_unique: false,
        default_value: null,
        calculation_expression: null,
        relation_target_entity:
          input.fieldType === "relation" ? input.relationTargetEntity.trim() : null,
        option_set_logical_name: null,
      };

      const response = await apiFetch(
        `/api/entities/${encodeURIComponent(entityLogicalName)}/fields`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );
      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(body.message ?? "Unable to create field.");
        return false;
      }

      const [fieldsRes, schemaRes] = await Promise.all([
        apiFetch(`/api/entities/${encodeURIComponent(entityLogicalName)}/fields`),
        apiFetch(`/api/entities/${encodeURIComponent(entityLogicalName)}/published`),
      ]);

      if (fieldsRes.ok) {
        const fields = (await fieldsRes.json()) as FieldResponse[];
        setFieldsCache((prev) => new Map(prev).set(entityLogicalName, fields));
      }
      if (schemaRes.ok) {
        const schema = (await schemaRes.json()) as PublishedSchemaResponse;
        setPublishedSchemaCache((prev) => new Map(prev).set(entityLogicalName, schema));
      }

      setStatusMessage(`Field '${logicalName}' created on '${entityLogicalName}'.`);
      return true;
    } catch {
      setErrorMessage("Unable to create field.");
      return false;
    } finally {
      setIsSaving(false);
    }
  }

  async function createForm(
    entityLogicalName: string,
    input: {
      logicalName: string;
      displayName: string;
      formType: "main" | "quick_create" | "quick_view";
    },
  ): Promise<string | null> {
    const logicalName = input.logicalName.trim();
    const displayName = input.displayName.trim();
    if (!logicalName || !displayName) {
      setErrorMessage("Form logical name and display name are required.");
      return null;
    }

    setIsSaving(true);
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const payload: CreateFormRequest = {
        logical_name: logicalName,
        display_name: displayName,
        form_type: input.formType,
        tabs: [
          {
            logical_name: "tab_1",
            display_name: "Tab 1",
            position: 0,
            visible: true,
            sections: [
              {
                logical_name: "section_1_1",
                display_name: "Section 1",
                position: 0,
                visible: true,
                columns: 2,
                fields: [],
                subgrids: [],
              },
            ],
          },
        ],
        header_fields: [],
      };

      const response = await apiFetch(
        `/api/entities/${encodeURIComponent(entityLogicalName)}/forms`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(body.message ?? "Unable to create form.");
        return null;
      }

      const formsRes = await apiFetch(
        `/api/entities/${encodeURIComponent(entityLogicalName)}/forms`,
      );
      if (formsRes.ok) {
        const forms = (await formsRes.json()) as FormResponse[];
        setFormsCache((prev) => new Map(prev).set(entityLogicalName, forms));
      }

      const bindingSynced = await syncAppBindingForEntity(entityLogicalName, {
        preferredFormLogicalName: logicalName,
      });
      setStatusMessage(
        bindingSynced
          ? `Form '${logicalName}' created and app binding synced.`
          : `Form '${logicalName}' created on '${entityLogicalName}'.`,
      );
      return logicalName;
    } catch {
      setErrorMessage("Unable to create form.");
      return null;
    } finally {
      setIsSaving(false);
    }
  }

  async function createView(
    entityLogicalName: string,
    input: {
      logicalName: string;
      displayName: string;
      viewType: string;
    },
  ): Promise<string | null> {
    const logicalName = input.logicalName.trim();
    const displayName = input.displayName.trim();
    if (!logicalName || !displayName) {
      setErrorMessage("View logical name and display name are required.");
      return null;
    }

    const fields = fieldsCache.get(entityLogicalName) ?? [];
    if (fields.length === 0) {
      setErrorMessage("Create at least one field before creating a view.");
      return null;
    }

    setIsSaving(true);
    setErrorMessage(null);
    setStatusMessage(null);
    try {
      const payload: CreateViewRequest = {
        logical_name: logicalName,
        display_name: displayName,
        view_type: input.viewType,
        columns: [
          {
            field_logical_name: fields[0].logical_name,
            position: 0,
            width: null,
            label_override: null,
          },
        ],
        default_sort: null,
        filter_criteria: null,
        is_default: false,
      };

      const response = await apiFetch(
        `/api/entities/${encodeURIComponent(entityLogicalName)}/views`,
        {
          method: "POST",
          body: JSON.stringify(payload),
        },
      );

      if (!response.ok) {
        const body = (await response.json()) as { message?: string };
        setErrorMessage(body.message ?? "Unable to create view.");
        return null;
      }

      const viewsRes = await apiFetch(
        `/api/entities/${encodeURIComponent(entityLogicalName)}/views`,
      );
      if (viewsRes.ok) {
        const views = (await viewsRes.json()) as ViewResponse[];
        setViewsCache((prev) => new Map(prev).set(entityLogicalName, views));
      }

      const bindingSynced = await syncAppBindingForEntity(entityLogicalName, {
        preferredViewLogicalName: logicalName,
      });
      setStatusMessage(
        bindingSynced
          ? `View '${logicalName}' created and app binding synced.`
          : `View '${logicalName}' created on '${entityLogicalName}'.`,
      );
      return logicalName;
    } catch {
      setErrorMessage("Unable to create view.");
      return null;
    } finally {
      setIsSaving(false);
    }
  }

  // ---- Set selection (with side-effects for loading form data) ----

  const setSelection = useCallback(
    (next: StudioSelection) => {
      setSelectionRaw(next);
      setErrorMessage(null);
      setStatusMessage(null);

      if (next.kind === "form") {
        const forms = formsCache.get(next.entityLogicalName) ?? [];
        const form = forms.find((f) => f.logical_name === next.formLogicalName) ?? null;
        setActiveFormResponse(form);
        formEditor.resetFromTabs(form?.tabs);
        setFormDisplayName(form?.display_name ?? "");
        setFormType(form?.form_type ?? "main");
        setHeaderFieldsText(form?.header_fields.join(", ") ?? "");
        setActiveViewResponse(null);
      } else if (next.kind === "view") {
        const views = viewsCache.get(next.entityLogicalName) ?? [];
        const view = views.find((v) => v.logical_name === next.viewLogicalName) ?? null;
        setActiveViewResponse(view);
        viewEditor.resetFromView({
          columns: view?.columns,
          default_sort: view?.default_sort,
          filter_criteria: view?.filter_criteria,
        });
        setViewDisplayName(view?.display_name ?? "");
        setViewType(view?.view_type ?? "grid");
        setViewIsDefault(view?.is_default ?? false);
        setActiveFormResponse(null);
      } else {
        setActiveFormResponse(null);
        setActiveViewResponse(null);
      }
    },
    [formEditor, formsCache, viewEditor, viewsCache],
  );

  // ---- Form save ----

  async function handleSaveForm(): Promise<void> {
    if (selection.kind !== "form") return;

    const schema = publishedSchemaCache.get(selection.entityLogicalName);
    if (!schema) {
      setErrorMessage("Publish the entity schema before saving forms.");
      return;
    }

    setIsSaving(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const payload: CreateFormRequest = {
        logical_name: selection.formLogicalName,
        display_name: formDisplayName,
        form_type: formType,
        tabs: formEditor?.tabs as unknown[],
        header_fields: normalizeHeaderFields(headerFieldsText),
      };

      const isEdit = activeFormResponse !== null;
      const path = isEdit
        ? `/api/entities/${encodeURIComponent(selection.entityLogicalName)}/forms/${encodeURIComponent(selection.formLogicalName)}`
        : `/api/entities/${encodeURIComponent(selection.entityLogicalName)}/forms`;

      const response = await apiFetch(path, {
        method: isEdit ? "PUT" : "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const data = (await response.json()) as { message?: string };
        setErrorMessage(data.message ?? "Unable to save form.");
        return;
      }

      // Refresh forms cache
      const formsRes = await apiFetch(
        `/api/entities/${encodeURIComponent(selection.entityLogicalName)}/forms`,
      );
      if (formsRes.ok) {
        const forms = (await formsRes.json()) as FormResponse[];
        setFormsCache((prev) => new Map(prev).set(selection.entityLogicalName, forms));
      }

      const bindingSynced = await syncAppBindingForEntity(selection.entityLogicalName, {
        preferredFormLogicalName: selection.formLogicalName,
      });
      setStatusMessage(
        bindingSynced ? "Form saved and app binding synced." : "Form saved.",
      );
    } catch {
      setErrorMessage("Unable to save form.");
    } finally {
      setIsSaving(false);
    }
  }

  async function handleSaveView(): Promise<void> {
    if (selection.kind !== "view") return;

    const schema = publishedSchemaCache.get(selection.entityLogicalName);
    if (!schema) {
      setErrorMessage("Publish the entity schema before saving views.");
      return;
    }

    if (viewEditor.columns.length === 0) {
      setErrorMessage("Add at least one column before saving view.");
      return;
    }

    setIsSaving(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const payload: CreateViewRequest = {
        logical_name: selection.viewLogicalName,
        display_name: viewDisplayName,
        view_type: viewType,
        columns: viewEditor.columns as unknown[],
        default_sort: viewEditor.defaultSort as unknown | null,
        filter_criteria:
          viewEditor.filterGroup && viewEditor.filterGroup.conditions.length > 0
            ? (viewEditor.filterGroup as unknown)
            : null,
        is_default: viewIsDefault,
      };

      const isEdit = activeViewResponse !== null;
      const path = isEdit
        ? `/api/entities/${encodeURIComponent(selection.entityLogicalName)}/views/${encodeURIComponent(selection.viewLogicalName)}`
        : `/api/entities/${encodeURIComponent(selection.entityLogicalName)}/views`;

      const response = await apiFetch(path, {
        method: isEdit ? "PUT" : "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const data = (await response.json()) as { message?: string };
        setErrorMessage(data.message ?? "Unable to save view.");
        return;
      }

      const viewsRes = await apiFetch(
        `/api/entities/${encodeURIComponent(selection.entityLogicalName)}/views`,
      );
      if (viewsRes.ok) {
        const views = (await viewsRes.json()) as ViewResponse[];
        setViewsCache((prev) => new Map(prev).set(selection.entityLogicalName, views));
      }

      const bindingSynced = await syncAppBindingForEntity(selection.entityLogicalName, {
        preferredViewLogicalName: selection.viewLogicalName,
      });
      setStatusMessage(
        bindingSynced ? "View saved and app binding synced." : "View saved.",
      );
    } catch {
      setErrorMessage("Unable to save view.");
    } finally {
      setIsSaving(false);
    }
  }

  // ---- Accessors ----

  const selectedAppDisplayName =
    apps.find((a) => a.logical_name === selectedApp)?.display_name ?? selectedApp;

  const getEntityFields = useCallback(
    (entityLogicalName: string) => fieldsCache.get(entityLogicalName) ?? [],
    [fieldsCache],
  );

  const getPublishedSchema = useCallback(
    (entityLogicalName: string) => publishedSchemaCache.get(entityLogicalName) ?? null,
    [publishedSchemaCache],
  );

  const formMeta =
    selection.kind === "form"
      ? {
          logicalName: selection.formLogicalName,
          displayName: formDisplayName,
          formType: formType,
          headerFieldsText: headerFieldsText,
        }
      : null;

  const viewMeta =
    selection.kind === "view"
      ? {
          logicalName: selection.viewLogicalName,
          displayName: viewDisplayName,
          viewType,
          isDefault: viewIsDefault,
        }
      : null;

  const setFormMeta = useCallback(
    (patch: Partial<{ displayName: string; formType: string; headerFieldsText: string }>) => {
      if (patch.displayName !== undefined) setFormDisplayName(patch.displayName);
      if (patch.formType !== undefined) setFormType(patch.formType);
      if (patch.headerFieldsText !== undefined) setHeaderFieldsText(patch.headerFieldsText);
    },
    [],
  );

  const setViewMeta = useCallback(
    (patch: Partial<{ displayName: string; viewType: string; isDefault: boolean }>) => {
      if (patch.displayName !== undefined) setViewDisplayName(patch.displayName);
      if (patch.viewType !== undefined) setViewType(patch.viewType);
      if (patch.isDefault !== undefined) setViewIsDefault(patch.isDefault);
    },
    [],
  );

  return {
    apps,
    entities: entitiesState,
    roles,
    selectedApp,
    setSelectedApp,
    selectedAppDisplayName,

    selection,
    setSelection,

    entityTree,
    isLoadingEntity,
    expandedEntities,
    toggleEntityExpanded,
    createEntity,
    createField,
    createForm,
    createView,

    getEntityFields,
    getPublishedSchema,
    refreshEntityPreviewRecords,

    formEditor: selection.kind === "form" ? formEditor : null,
    formMeta,
    setFormMeta,
    viewEditor: selection.kind === "view" ? viewEditor : null,
    viewMeta,
    setViewMeta,

    isSaving,
    errorMessage,
    statusMessage,
    clearMessages: () => {
      setErrorMessage(null);
      setStatusMessage(null);
    },
    handleSaveForm,
    handleSaveView,
  };
}
