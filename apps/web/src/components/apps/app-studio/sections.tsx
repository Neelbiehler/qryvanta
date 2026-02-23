import { type DragEvent, type FormEvent, useMemo } from "react";
import Link from "next/link";

import {
  Button,
  Checkbox,
  Input,
  Label,
  Select,
  StatusBadge,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  buttonVariants,
} from "@qryvanta/ui";

import {
  type AppEntityBindingResponse,
  type AppResponse,
  type AppRoleEntityPermissionResponse,
  type EntityResponse,
  type FieldResponse,
  type RoleResponse,
} from "@/lib/api";
import { cn } from "@/lib/utils";

export type AppEntityViewMode = "grid" | "json";
export type AppStudioSection = "apps" | "navigation" | "permissions";

export type NewAppDraft = {
  logicalName: string;
  displayName: string;
  description: string;
};

export type BindingDraft = {
  entityToBind: string;
  navigationLabel: string;
  navigationOrder: number;
  forms: AppSurfaceDraft[];
  listViews: AppSurfaceDraft[];
  defaultFormLogicalName: string;
  defaultListViewLogicalName: string;
  defaultViewMode: AppEntityViewMode;
};

export type AppSurfaceDraft = {
  logicalName: string;
  displayName: string;
  fieldLogicalNames: string[];
};

export type PermissionDraft = {
  roleName: string;
  entityName: string;
  canRead: boolean;
  canCreate: boolean;
  canUpdate: boolean;
  canDelete: boolean;
};

type AppStudioOverviewProps = {
  activeSection: AppStudioSection;
  appsCount: number;
  canOpenNavigation: boolean;
  canOpenPermissions: boolean;
  entitiesCount: number;
  hasStudioData: boolean;
  onSectionChange: (section: AppStudioSection) => void;
  rolesCount: number;
  selectedAppDisplayName: string;
};

export function AppStudioOverview({
  activeSection,
  appsCount,
  canOpenNavigation,
  canOpenPermissions,
  entitiesCount,
  hasStudioData,
  onSectionChange,
  rolesCount,
  selectedAppDisplayName,
}: AppStudioOverviewProps) {
  return (
    <>
      {!hasStudioData ? (
        <p className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-800">
          Create at least one app, one entity, and one role before configuring
          app access.
        </p>
      ) : null}

      <div className="flex flex-wrap items-center gap-2 rounded-md border border-emerald-100 bg-white/90 p-3">
        <StatusBadge tone="neutral">Apps {appsCount}</StatusBadge>
        <StatusBadge tone="neutral">Entities {entitiesCount}</StatusBadge>
        <StatusBadge tone="neutral">Roles {rolesCount}</StatusBadge>
        <StatusBadge tone="success">Active {selectedAppDisplayName}</StatusBadge>
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant={activeSection === "apps" ? "default" : "outline"}
          onClick={() => onSectionChange("apps")}
        >
          App Catalog
        </Button>
        <Button
          type="button"
          variant={activeSection === "navigation" ? "default" : "outline"}
          onClick={() => onSectionChange("navigation")}
          disabled={!canOpenNavigation}
        >
          Navigation Binding
        </Button>
        <Button
          type="button"
          variant={activeSection === "permissions" ? "default" : "outline"}
          onClick={() => onSectionChange("permissions")}
          disabled={!canOpenPermissions}
        >
          Role Permissions
        </Button>
      </div>
    </>
  );
}

type AppCatalogSectionProps = {
  apps: AppResponse[];
  isCreatingApp: boolean;
  newAppDraft: NewAppDraft;
  onCreateApp: (event: FormEvent<HTMLFormElement>) => void;
  onUpdateDraft: (next: NewAppDraft) => void;
};

export function AppCatalogSection({
  apps,
  isCreatingApp,
  newAppDraft,
  onCreateApp,
  onUpdateDraft,
}: AppCatalogSectionProps) {
  return (
    <div className="space-y-3 rounded-md border border-zinc-200 bg-white p-4">
      <div>
        <p className="text-sm font-semibold text-zinc-900">App Catalog</p>
        <p className="text-xs text-zinc-600">
          Define application shells before sitemap and role matrix configuration.
        </p>
      </div>

      <form className="grid gap-3 md:grid-cols-3" onSubmit={onCreateApp}>
        <div className="space-y-2">
          <Label htmlFor="new_app_logical_name">App Logical Name</Label>
          <Input
            id="new_app_logical_name"
            value={newAppDraft.logicalName}
            onChange={(event) =>
              onUpdateDraft({ ...newAppDraft, logicalName: event.target.value })
            }
            placeholder="sales"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="new_app_display_name">App Display Name</Label>
          <Input
            id="new_app_display_name"
            value={newAppDraft.displayName}
            onChange={(event) =>
              onUpdateDraft({ ...newAppDraft, displayName: event.target.value })
            }
            placeholder="Sales App"
            required
          />
        </div>
        <div className="space-y-2">
          <Label htmlFor="new_app_description">Description</Label>
          <Input
            id="new_app_description"
            value={newAppDraft.description}
            onChange={(event) =>
              onUpdateDraft({ ...newAppDraft, description: event.target.value })
            }
            placeholder="Lead and account workflows"
          />
        </div>
        <div className="md:col-span-3">
          <Button disabled={isCreatingApp} type="submit">
            {isCreatingApp ? "Creating..." : "Create App"}
          </Button>
        </div>
      </form>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>App</TableHead>
            <TableHead>Description</TableHead>
            <TableHead>Logical Name</TableHead>
            <TableHead className="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {apps.length > 0 ? (
            apps.map((app) => (
              <TableRow key={app.logical_name}>
                <TableCell>{app.display_name}</TableCell>
                <TableCell>{app.description ?? "-"}</TableCell>
                <TableCell className="font-mono text-xs">
                  {app.logical_name}
                </TableCell>
                <TableCell className="text-right">
                  <Link
                    href={`/maker/apps/${encodeURIComponent(app.logical_name)}/sitemap`}
                    className={cn(buttonVariants({ size: "sm", variant: "outline" }))}
                  >
                    Open Sitemap
                  </Link>
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={4} className="text-zinc-500">
                No apps yet.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

type NavigationBindingSectionProps = {
  apps: AppResponse[];
  bindings: AppEntityBindingResponse[];
  entities: EntityResponse[];
  selectedEntityFields: FieldResponse[];
  isLoadingEntityFields: boolean;
  isBindingEntity: boolean;
  isLoadingAppData: boolean;
  onBindEntity: (event: FormEvent<HTMLFormElement>) => void;
  onChangeSelectedApp: (appLogicalName: string) => void;
  onUpdateBindingDraft: (next: BindingDraft) => void;
  selectedApp: string;
  selectedAppDisplayName: string;
  bindingDraft: BindingDraft;
};

export function NavigationBindingSection({
  apps,
  bindings,
  entities,
  selectedEntityFields,
  isLoadingEntityFields,
  isBindingEntity,
  isLoadingAppData,
  onBindEntity,
  onChangeSelectedApp,
  onUpdateBindingDraft,
  selectedApp,
  selectedAppDisplayName,
  bindingDraft,
}: NavigationBindingSectionProps) {
  return (
    <div className="space-y-3 rounded-md border border-zinc-200 bg-white p-4">
      <div>
        <p className="text-sm font-semibold text-zinc-900">Sitemap Navigation</p>
        <p className="text-xs text-zinc-600">
          Bind entities to the app sitemap and configure list/form presentation defaults.
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="studio_app_selector">Active App</Label>
        <Select
          id="studio_app_selector"
          value={selectedApp}
          onChange={(event) => onChangeSelectedApp(event.target.value)}
        >
          {apps.map((app) => (
            <option key={app.logical_name} value={app.logical_name}>
              {app.display_name} ({app.logical_name})
            </option>
          ))}
        </Select>
      </div>

      <form className="grid gap-3 md:grid-cols-2" onSubmit={onBindEntity}>
        <div className="space-y-2">
          <Label htmlFor="bind_entity_name">Entity</Label>
          <Select
            id="bind_entity_name"
            value={bindingDraft.entityToBind}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                entityToBind: event.target.value,
              })
            }
          >
            {entities.map((entity) => (
              <option key={entity.logical_name} value={entity.logical_name}>
                {entity.display_name} ({entity.logical_name})
              </option>
            ))}
          </Select>
          <p className="text-[11px] text-zinc-500">
            {isLoadingEntityFields
              ? "Loading entity fields..."
              : `${selectedEntityFields.length} field(s) available for form/view design.`}
          </p>
        </div>

        <div className="space-y-2">
          <Label htmlFor="bind_navigation_label">Navigation Label</Label>
          <Input
            id="bind_navigation_label"
            value={bindingDraft.navigationLabel}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                navigationLabel: event.target.value,
              })
            }
            placeholder="Accounts"
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="bind_navigation_order">Navigation Order</Label>
          <Input
            id="bind_navigation_order"
            value={String(bindingDraft.navigationOrder)}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                navigationOrder: Number.parseInt(event.target.value || "0", 10),
              })
            }
            type="number"
            min={0}
          />
        </div>

        <div className="md:col-span-2">
          <FieldLayoutDesigner
            selectedEntityFields={selectedEntityFields}
            forms={bindingDraft.forms}
            listViews={bindingDraft.listViews}
            defaultFormLogicalName={bindingDraft.defaultFormLogicalName}
            defaultListViewLogicalName={bindingDraft.defaultListViewLogicalName}
            onChangeForms={(forms) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                forms,
              })
            }
            onChangeListViews={(listViews) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                listViews,
              })
            }
            onChangeDefaultFormLogicalName={(defaultFormLogicalName) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                defaultFormLogicalName,
              })
            }
            onChangeDefaultListViewLogicalName={(defaultListViewLogicalName) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                defaultListViewLogicalName,
              })
            }
          />
        </div>

        <div className="space-y-2">
          <Label htmlFor="bind_default_view_mode">Default View Mode</Label>
          <Select
            id="bind_default_view_mode"
            value={bindingDraft.defaultViewMode}
            onChange={(event) =>
              onUpdateBindingDraft({
                ...bindingDraft,
                defaultViewMode: event.target.value as AppEntityViewMode,
              })
            }
          >
            <option value="grid">Grid</option>
            <option value="json">JSON</option>
          </Select>
        </div>

        <div className="md:col-span-2">
          <Button
            disabled={isBindingEntity || isLoadingAppData}
            type="submit"
            variant="outline"
          >
            {isBindingEntity ? "Saving..." : `Bind Entity to ${selectedAppDisplayName}`}
          </Button>
        </div>
      </form>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Bound Entity</TableHead>
            <TableHead>Label</TableHead>
            <TableHead>Order</TableHead>
            <TableHead>Default View</TableHead>
            <TableHead>Presentation</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {bindings.length > 0 ? (
            bindings.map((binding) => (
              <TableRow
                key={`${binding.app_logical_name}.${binding.entity_logical_name}`}
              >
                <TableCell className="font-mono text-xs">
                  {binding.entity_logical_name}
                </TableCell>
                <TableCell>
                  {binding.navigation_label ?? binding.entity_logical_name}
                </TableCell>
                <TableCell>{binding.navigation_order}</TableCell>
                <TableCell className="uppercase">{binding.default_view_mode}</TableCell>
                <TableCell className="text-xs text-zinc-600">
                  {resolveBindingFormCount(binding)} form(s) / {resolveBindingViewCount(binding)} view(s)
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={5} className="text-zinc-500">
                No entity bindings for this app.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

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

function FieldLayoutDesigner({
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
    const nextForms = forms.map((form) =>
      form.logicalName === selectedForm.logicalName ? next : form,
    );
    onChangeForms(nextForms);
  }

  function updateListView(next: AppSurfaceDraft) {
    const nextViews = listViews.map((view) =>
      view.logicalName === selectedListView.logicalName ? next : view,
    );
    onChangeListViews(nextViews);
  }

  function onDragStart(
    event: DragEvent<HTMLButtonElement>,
    logicalName: string,
    source: "available" | "form" | "view",
  ) {
    event.dataTransfer.setData("text/app-field-logical-name", logicalName);
    event.dataTransfer.setData("text/app-field-source", source);
    event.dataTransfer.effectAllowed = "move";
  }

  function onDropToTarget(event: DragEvent<HTMLDivElement>, target: "form" | "view") {
    event.preventDefault();
    const logicalName = event.dataTransfer.getData("text/app-field-logical-name").trim();
    if (!logicalName) {
      return;
    }

    if (target === "form") {
      updateForm({
        ...selectedForm,
        fieldLogicalNames: appendUniqueField(selectedForm.fieldLogicalNames, logicalName),
      });
    } else {
      updateListView({
        ...selectedListView,
        fieldLogicalNames: appendUniqueField(selectedListView.fieldLogicalNames, logicalName),
      });
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

  function renameForm(
    formLogicalName: string,
    key: "logicalName" | "displayName",
    value: string,
  ) {
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

      <div className="grid gap-3 md:grid-cols-2">
        <div className="space-y-2 rounded-md border border-zinc-200 bg-white p-2">
          <div className="flex items-center justify-between gap-2">
            <Label htmlFor="bind_default_form">Default Form</Label>
            <Button type="button" size="sm" variant="outline" onClick={() => createSurface("form")}>
              Add Form
            </Button>
          </div>
          <Select
            id="bind_default_form"
            value={selectedForm.logicalName}
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
            <Button type="button" size="sm" variant="outline" onClick={() => createSurface("view")}>
              Add View
            </Button>
          </div>
          <Select
            id="bind_default_list_view"
            value={selectedListView.logicalName}
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

      <div className="grid gap-3 md:grid-cols-2">
        <div className="space-y-2 rounded-md border border-zinc-200 bg-white p-2">
          <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500">
            Form Catalog
          </p>
          <div className="space-y-2">
            {forms.map((form) => (
              <div key={`form-catalog-${form.logicalName}`} className="rounded-md border border-zinc-200 p-2">
                <div className="grid gap-2 md:grid-cols-[1fr_1fr_auto_auto]">
                  <Input
                    value={form.displayName}
                    onChange={(event) =>
                      renameForm(form.logicalName, "displayName", event.target.value)
                    }
                    placeholder="Display Name"
                  />
                  <Input
                    value={form.logicalName}
                    onChange={(event) =>
                      renameForm(form.logicalName, "logicalName", event.target.value)
                    }
                    placeholder="logical_name"
                  />
                  <Button
                    type="button"
                    size="sm"
                    variant={form.logicalName === defaultFormLogicalName ? "default" : "outline"}
                    onClick={() => onChangeDefaultFormLogicalName(form.logicalName)}
                  >
                    {form.logicalName === defaultFormLogicalName ? "Default" : "Make Default"}
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => deleteForm(form.logicalName)}
                    disabled={forms.length <= 1 || form.logicalName === defaultFormLogicalName}
                  >
                    Delete
                  </Button>
                </div>
              </div>
            ))}
          </div>
          <p className="text-[11px] text-zinc-500">
            Keep at least one form. Switch default before deleting the active default.
          </p>
        </div>

        <div className="space-y-2 rounded-md border border-zinc-200 bg-white p-2">
          <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500">
            View Catalog
          </p>
          <div className="space-y-2">
            {listViews.map((view) => (
              <div key={`view-catalog-${view.logicalName}`} className="rounded-md border border-zinc-200 p-2">
                <div className="grid gap-2 md:grid-cols-[1fr_1fr_auto_auto]">
                  <Input
                    value={view.displayName}
                    onChange={(event) =>
                      renameListView(view.logicalName, "displayName", event.target.value)
                    }
                    placeholder="Display Name"
                  />
                  <Input
                    value={view.logicalName}
                    onChange={(event) =>
                      renameListView(view.logicalName, "logicalName", event.target.value)
                    }
                    placeholder="logical_name"
                  />
                  <Button
                    type="button"
                    size="sm"
                    variant={view.logicalName === defaultListViewLogicalName ? "default" : "outline"}
                    onClick={() => onChangeDefaultListViewLogicalName(view.logicalName)}
                  >
                    {view.logicalName === defaultListViewLogicalName ? "Default" : "Make Default"}
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => deleteListView(view.logicalName)}
                    disabled={listViews.length <= 1 || view.logicalName === defaultListViewLogicalName}
                  >
                    Delete
                  </Button>
                </div>
              </div>
            ))}
          </div>
          <p className="text-[11px] text-zinc-500">
            Keep at least one list view. Switch default before deleting the active default.
          </p>
        </div>
      </div>

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
            updateForm({
              ...selectedForm,
              fieldLogicalNames: appendUniqueField(
                selectedForm.fieldLogicalNames,
                logicalName,
              ),
            })
          }
          onMoveField={(logicalName, direction) =>
            updateForm({
              ...selectedForm,
              fieldLogicalNames: moveField(
                selectedForm.fieldLogicalNames,
                logicalName,
                direction,
              ),
            })
          }
          onRemoveField={(logicalName) =>
            updateForm({
              ...selectedForm,
              fieldLogicalNames: selectedForm.fieldLogicalNames.filter(
                (field) => field !== logicalName,
              ),
            })
          }
          onDrop={(event) => onDropToTarget(event, "form")}
          onDragStart={onDragStart}
        />

        <FieldDropZone
          title={selectedListView.displayName}
          helperText="Grid/list columns"
          dragSource="view"
          fieldLogicalNames={selectedListView.fieldLogicalNames}
          fieldLabelByLogicalName={fieldLabelByLogicalName}
          onAddField={(logicalName) =>
            updateListView({
              ...selectedListView,
              fieldLogicalNames: appendUniqueField(
                selectedListView.fieldLogicalNames,
                logicalName,
              ),
            })
          }
          onMoveField={(logicalName, direction) =>
            updateListView({
              ...selectedListView,
              fieldLogicalNames: moveField(
                selectedListView.fieldLogicalNames,
                logicalName,
                direction,
              ),
            })
          }
          onRemoveField={(logicalName) =>
            updateListView({
              ...selectedListView,
              fieldLogicalNames: selectedListView.fieldLogicalNames.filter(
                (field) => field !== logicalName,
              ),
            })
          }
          onDrop={(event) => onDropToTarget(event, "view")}
          onDragStart={onDragStart}
        />
      </div>

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
                updateForm({
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
                updateListView({
                  ...selectedListView,
                  fieldLogicalNames: parseFieldLogicalNames(event.target.value),
                })
              }
              placeholder="name, status, updated_at"
            />
          </div>
        </div>
      </details>
    </div>
  );
}

type FieldDropZoneProps = {
  title: string;
  helperText: string;
  dragSource: "form" | "view";
  fieldLogicalNames: string[];
  fieldLabelByLogicalName: Map<string, string>;
  onAddField: (logicalName: string) => void;
  onMoveField: (logicalName: string, direction: "up" | "down") => void;
  onRemoveField: (logicalName: string) => void;
  onDrop: (event: DragEvent<HTMLDivElement>) => void;
  onDragStart: (
    event: DragEvent<HTMLButtonElement>,
    logicalName: string,
    source: "available" | "form" | "view",
  ) => void;
};

function FieldDropZone({
  title,
  helperText,
  dragSource,
  fieldLogicalNames,
  fieldLabelByLogicalName,
  onAddField,
  onMoveField,
  onRemoveField,
  onDrop,
  onDragStart,
}: FieldDropZoneProps) {
  return (
    <div
      className="space-y-2 rounded-md border border-zinc-200 bg-white p-2"
      onDragOver={(event) => {
        event.preventDefault();
        event.dataTransfer.dropEffect = "move";
      }}
      onDrop={onDrop}
    >
      <p className="text-xs font-semibold uppercase tracking-wide text-zinc-500">{title}</p>
      <p className="text-[11px] text-zinc-500">{helperText}</p>

      <div className="max-h-48 space-y-1 overflow-y-auto pr-1">
        {fieldLogicalNames.length > 0 ? (
          fieldLogicalNames.map((logicalName, index) => {
            const label = fieldLabelByLogicalName.get(logicalName) ?? logicalName;
            return (
              <div
                key={`${title}-${logicalName}-${index}`}
                className="rounded-md border border-zinc-200 bg-zinc-50 px-2 py-1"
              >
                <button
                  type="button"
                  draggable
                  onDragStart={(event) => onDragStart(event, logicalName, dragSource)}
                  className="w-full text-left"
                >
                  <p className="text-xs font-medium text-zinc-900">{label}</p>
                </button>
                <div className="mt-1 flex flex-wrap gap-1">
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => onMoveField(logicalName, "up")}
                    disabled={index === 0}
                  >
                    Up
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => onMoveField(logicalName, "down")}
                    disabled={index === fieldLogicalNames.length - 1}
                  >
                    Down
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    onClick={() => onRemoveField(logicalName)}
                  >
                    Remove
                  </Button>
                </div>
              </div>
            );
          })
        ) : (
          <p className="text-[11px] text-zinc-500">Drop fields here.</p>
        )}
      </div>

      <Select
        value=""
        onChange={(event) => {
          const logicalName = event.target.value;
          if (!logicalName) {
            return;
          }
          onAddField(logicalName);
        }}
      >
        <option value="">Add field...</option>
        {Array.from(fieldLabelByLogicalName.keys()).map((logicalName) => (
          <option key={`${title}-select-${logicalName}`} value={logicalName}>
            {fieldLabelByLogicalName.get(logicalName)}
          </option>
        ))}
      </Select>
    </div>
  );
}

function parseFieldLogicalNames(raw: string): string[] {
  return raw
    .split(",")
    .map((value) => value.trim())
    .filter((value) => value.length > 0);
}

function serializeFieldLogicalNames(fields: string[]): string {
  return fields.join(", ");
}

function appendUniqueField(fields: string[], logicalName: string): string[] {
  return fields.includes(logicalName) ? fields : [...fields, logicalName];
}

function moveField(
  fields: string[],
  logicalName: string,
  direction: "up" | "down",
): string[] {
  const index = fields.indexOf(logicalName);
  if (index < 0) {
    return fields;
  }

  const targetIndex = direction === "up" ? index - 1 : index + 1;
  if (targetIndex < 0 || targetIndex >= fields.length) {
    return fields;
  }

  const next = [...fields];
  const [entry] = next.splice(index, 1);
  next.splice(targetIndex, 0, entry);
  return next;
}

function buildNextLogicalName(
  existingLogicalNames: string[],
  prefix: string,
  startIndex: number,
): string {
  let nextIndex = startIndex;
  let candidate = `${prefix}_${nextIndex}`;

  while (existingLogicalNames.includes(candidate)) {
    nextIndex += 1;
    candidate = `${prefix}_${nextIndex}`;
  }

  return candidate;
}

function normalizeSurfaceLogicalName(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/\s+/g, "_")
    .replace(/[^a-z0-9_]/g, "");
}

function resolveBindingFormCount(binding: AppEntityBindingResponse): number {
  if (binding.forms.length > 0) {
    return binding.forms.length;
  }

  return binding.form_field_logical_names.length > 0 ? 1 : 0;
}

function resolveBindingViewCount(binding: AppEntityBindingResponse): number {
  if (binding.list_views.length > 0) {
    return binding.list_views.length;
  }

  return binding.list_field_logical_names.length > 0 ? 1 : 0;
}

type RolePermissionsSectionProps = {
  apps: AppResponse[];
  entities: EntityResponse[];
  isLoadingAppData: boolean;
  isSavingPermission: boolean;
  onChangeSelectedApp: (appLogicalName: string) => void;
  onSavePermission: (event: FormEvent<HTMLFormElement>) => void;
  onUpdatePermissionDraft: (next: PermissionDraft) => void;
  permissions: AppRoleEntityPermissionResponse[];
  roles: RoleResponse[];
  selectedApp: string;
  permissionDraft: PermissionDraft;
};

export function RolePermissionsSection({
  apps,
  entities,
  isLoadingAppData,
  isSavingPermission,
  onChangeSelectedApp,
  onSavePermission,
  onUpdatePermissionDraft,
  permissions,
  roles,
  selectedApp,
  permissionDraft,
}: RolePermissionsSectionProps) {
  return (
    <div className="space-y-3 rounded-md border border-zinc-200 bg-white p-4">
      <div>
        <p className="text-sm font-semibold text-zinc-900">Role Matrix</p>
        <p className="text-xs text-zinc-600">
          Configure per-role CRUD permissions for each app entity.
        </p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="studio_permissions_app_selector">Active App</Label>
        <Select
          id="studio_permissions_app_selector"
          value={selectedApp}
          onChange={(event) => onChangeSelectedApp(event.target.value)}
        >
          {apps.map((app) => (
            <option key={app.logical_name} value={app.logical_name}>
              {app.display_name} ({app.logical_name})
            </option>
          ))}
        </Select>
      </div>

      <form className="grid gap-3 md:grid-cols-4" onSubmit={onSavePermission}>
        <div className="space-y-2">
          <Label htmlFor="permission_role_name">Role</Label>
          <Select
            id="permission_role_name"
            value={permissionDraft.roleName}
            onChange={(event) =>
              onUpdatePermissionDraft({
                ...permissionDraft,
                roleName: event.target.value,
              })
            }
          >
            {roles.map((role) => (
              <option key={role.role_id} value={role.name}>
                {role.name}
              </option>
            ))}
          </Select>
        </div>

        <div className="space-y-2">
          <Label htmlFor="permission_entity_name">Entity</Label>
          <Select
            id="permission_entity_name"
            value={permissionDraft.entityName}
            onChange={(event) =>
              onUpdatePermissionDraft({
                ...permissionDraft,
                entityName: event.target.value,
              })
            }
          >
            {entities.map((entity) => (
              <option key={entity.logical_name} value={entity.logical_name}>
                {entity.display_name} ({entity.logical_name})
              </option>
            ))}
          </Select>
        </div>

        <div className="space-y-1 pt-6 md:col-span-2">
          <div className="mr-3 inline-flex items-center gap-1 text-sm">
            <Checkbox
              id="permission_can_read"
              checked={permissionDraft.canRead}
              onChange={(event) =>
                onUpdatePermissionDraft({
                  ...permissionDraft,
                  canRead: event.target.checked,
                })
              }
            />
            <Label htmlFor="permission_can_read">Read</Label>
          </div>
          <div className="mr-3 inline-flex items-center gap-1 text-sm">
            <Checkbox
              id="permission_can_create"
              checked={permissionDraft.canCreate}
              onChange={(event) =>
                onUpdatePermissionDraft({
                  ...permissionDraft,
                  canCreate: event.target.checked,
                })
              }
            />
            <Label htmlFor="permission_can_create">Create</Label>
          </div>
          <div className="mr-3 inline-flex items-center gap-1 text-sm">
            <Checkbox
              id="permission_can_update"
              checked={permissionDraft.canUpdate}
              onChange={(event) =>
                onUpdatePermissionDraft({
                  ...permissionDraft,
                  canUpdate: event.target.checked,
                })
              }
            />
            <Label htmlFor="permission_can_update">Update</Label>
          </div>
          <div className="inline-flex items-center gap-1 text-sm">
            <Checkbox
              id="permission_can_delete"
              checked={permissionDraft.canDelete}
              onChange={(event) =>
                onUpdatePermissionDraft({
                  ...permissionDraft,
                  canDelete: event.target.checked,
                })
              }
            />
            <Label htmlFor="permission_can_delete">Delete</Label>
          </div>
        </div>

        <div className="md:col-span-4">
          <Button
            disabled={isSavingPermission || isLoadingAppData}
            type="submit"
            variant="outline"
          >
            {isSavingPermission ? "Saving..." : "Save Role Entity Permissions"}
          </Button>
        </div>
      </form>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Role</TableHead>
            <TableHead>Entity</TableHead>
            <TableHead>Permissions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {permissions.length > 0 ? (
            permissions.map((permission) => (
              <TableRow
                key={`${permission.app_logical_name}.${permission.role_name}.${permission.entity_logical_name}`}
              >
                <TableCell>{permission.role_name}</TableCell>
                <TableCell className="font-mono text-xs">
                  {permission.entity_logical_name}
                </TableCell>
                <TableCell className="font-mono text-xs">
                  {[
                    permission.can_read ? "read" : null,
                    permission.can_create ? "create" : null,
                    permission.can_update ? "update" : null,
                    permission.can_delete ? "delete" : null,
                  ]
                    .filter((value): value is string => value !== null)
                    .join(", ") || "none"}
                </TableCell>
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={3} className="text-zinc-500">
                No role entity permissions configured for this app.
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}
