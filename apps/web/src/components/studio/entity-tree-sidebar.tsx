"use client";

import { useState } from "react";

import { Button } from "@qryvanta/ui";
import { Checkbox, Input, Label, Select } from "@qryvanta/ui";

import type { StudioController } from "@/components/studio/hooks/use-studio-state";
import type { EntityTreeNode, StudioSelection } from "@/components/studio/types";
import { cn } from "@/lib/utils";

type EntityTreeSidebarProps = {
  studio: StudioController;
};

export function EntityTreeSidebar({ studio }: EntityTreeSidebarProps) {
  const [entityLogicalName, setEntityLogicalName] = useState("");
  const [entityDisplayName, setEntityDisplayName] = useState("");
  const [isCreateEntityOpen, setIsCreateEntityOpen] = useState(false);

  return (
    <aside className="flex h-full min-h-0 flex-col overflow-y-auto rounded-xl border border-zinc-200 bg-white">
      <div className="border-b border-zinc-200 px-3 py-3">
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
          Solution Explorer
        </p>
        <p className="mt-0.5 text-[11px] text-zinc-500">
          {studio.selectedAppDisplayName}
        </p>
        <div className="mt-2">
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="h-7 w-full"
            onClick={() => setIsCreateEntityOpen((current) => !current)}
          >
            {isCreateEntityOpen ? "Close" : "+ Entity"}
          </Button>
        </div>
        {isCreateEntityOpen ? (
          <form
            className="mt-2 space-y-2 rounded-md border border-zinc-200 bg-zinc-50 p-2"
            onSubmit={(event) => {
              event.preventDefault();
              void studio
                .createEntity({
                  logicalName: entityLogicalName,
                  displayName: entityDisplayName,
                })
                .then((created) => {
                  if (!created) return;
                  setEntityLogicalName("");
                  setEntityDisplayName("");
                  setIsCreateEntityOpen(false);
                });
            }}
          >
            <div className="space-y-1">
              <Label htmlFor="studio_new_entity_logical" className="text-[11px]">
                Logical Name
              </Label>
              <Input
                id="studio_new_entity_logical"
                value={entityLogicalName}
                onChange={(event) => setEntityLogicalName(event.target.value)}
                placeholder="account"
                className="h-8 text-xs"
                required
              />
            </div>
            <div className="space-y-1">
              <Label htmlFor="studio_new_entity_display" className="text-[11px]">
                Display Name
              </Label>
              <Input
                id="studio_new_entity_display"
                value={entityDisplayName}
                onChange={(event) => setEntityDisplayName(event.target.value)}
                placeholder="Account"
                className="h-8 text-xs"
                required
              />
            </div>
            <Button type="submit" size="sm" disabled={studio.isSaving} className="w-full">
              {studio.isSaving ? "Creating..." : "Create Entity"}
            </Button>
          </form>
        ) : null}
      </div>

      <nav className="flex-1 space-y-0.5 overflow-y-auto p-2">
        <TreeButton
          label="Overview"
          active={studio.selection.kind === "overview"}
          onClick={() => studio.setSelection({ kind: "overview" })}
          indent={0}
        />

        <TreeButton
          label="Sitemap"
          active={studio.selection.kind === "sitemap"}
          onClick={() => studio.setSelection({ kind: "sitemap" })}
          indent={0}
        />

        <div className="pt-1">
          <p className="px-2 pb-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-zinc-400">
            Entities
          </p>
          {studio.entityTree.map((entity) => (
            <EntityNode
              key={entity.logicalName}
              entity={entity}
              isExpanded={studio.expandedEntities.has(entity.logicalName)}
              onToggle={() => studio.toggleEntityExpanded(entity.logicalName)}
              selection={studio.selection}
              onSelect={studio.setSelection}
              isLoading={studio.isLoadingEntity}
              onCreateField={studio.createField}
              onCreateForm={studio.createForm}
              onCreateView={studio.createView}
              allEntities={studio.entities}
              entityFields={studio.getEntityFields(entity.logicalName)}
              isSaving={studio.isSaving}
            />
          ))}
          {studio.entityTree.length === 0 ? (
            <p className="px-2 py-1 text-xs text-zinc-400">No entities yet.</p>
          ) : null}
        </div>

        <div className="pt-1">
          <TreeButton
            label="Security"
            active={studio.selection.kind === "security"}
            onClick={() => studio.setSelection({ kind: "security" })}
            indent={0}
          />
          <TreeButton
            label="Publish Console"
            active={studio.selection.kind === "publish"}
            onClick={() => studio.setSelection({ kind: "publish" })}
            indent={0}
          />
        </div>
      </nav>
    </aside>
  );
}

// ---------------------------------------------------------------------------
// Entity tree node
// ---------------------------------------------------------------------------

type EntityNodeProps = {
  entity: EntityTreeNode;
  isExpanded: boolean;
  onToggle: () => void;
  selection: StudioSelection;
  onSelect: (selection: StudioSelection) => void;
  isLoading: boolean;
  onCreateField: StudioController["createField"];
  onCreateForm: StudioController["createForm"];
  onCreateView: StudioController["createView"];
  allEntities: StudioController["entities"];
  entityFields: ReturnType<StudioController["getEntityFields"]>;
  isSaving: boolean;
};

function EntityNode({
  entity,
  isExpanded,
  onToggle,
  selection,
  onSelect,
  isLoading,
  onCreateField,
  onCreateForm,
  onCreateView,
  allEntities,
  entityFields,
  isSaving,
}: EntityNodeProps) {
  const [isCreateFieldOpen, setIsCreateFieldOpen] = useState(false);
  const [fieldLogicalName, setFieldLogicalName] = useState("");
  const [fieldDisplayName, setFieldDisplayName] = useState("");
  const [fieldType, setFieldType] = useState<
    "text" | "number" | "boolean" | "date" | "datetime" | "json" | "relation"
  >("text");
  const [relationTarget, setRelationTarget] = useState("");
  const [isRequired, setIsRequired] = useState(false);
  const [isCreateFormOpen, setIsCreateFormOpen] = useState(false);
  const [formLogicalName, setFormLogicalName] = useState("");
  const [formDisplayName, setFormDisplayName] = useState("");
  const [formType, setFormType] = useState<"main" | "quick_create" | "quick_view">("main");
  const [isCreateViewOpen, setIsCreateViewOpen] = useState(false);
  const [viewLogicalName, setViewLogicalName] = useState("");
  const [viewDisplayName, setViewDisplayName] = useState("");
  const [viewType, setViewType] = useState("grid");

  return (
    <div>
      <button
        type="button"
        className={cn(
          "flex w-full items-center gap-1.5 rounded-md px-2 py-1.5 text-left text-sm transition",
          "hover:bg-zinc-100",
          isExpanded && "font-medium text-zinc-900",
          !isExpanded && "text-zinc-700",
        )}
        onClick={onToggle}
      >
        <span className="text-[10px] text-zinc-400">{isExpanded ? "▼" : "▶"}</span>
        <span className="truncate">{entity.displayName}</span>
        <span className="ml-auto font-mono text-[10px] text-zinc-400">
          {entity.logicalName}
        </span>
      </button>

      {isExpanded ? (
        <div className="ml-3 border-l border-zinc-200 pl-1">
          <div className="px-2 pb-1 pt-1.5">
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-6 w-full text-[11px]"
              onClick={() => setIsCreateFieldOpen((current) => !current)}
            >
              {isCreateFieldOpen ? "Close" : "+ Field"}
            </Button>
          </div>

          {isCreateFieldOpen ? (
            <form
              className="mx-2 mb-1 space-y-1.5 rounded border border-zinc-200 bg-zinc-50 p-2"
              onSubmit={(event) => {
                event.preventDefault();
                void onCreateField(entity.logicalName, {
                  logicalName: fieldLogicalName,
                  displayName: fieldDisplayName,
                  fieldType,
                  isRequired,
                  relationTargetEntity: relationTarget,
                }).then((created) => {
                  if (!created) return;
                  setFieldLogicalName("");
                  setFieldDisplayName("");
                  setFieldType("text");
                  setRelationTarget("");
                  setIsRequired(false);
                  setIsCreateFieldOpen(false);
                });
              }}
            >
              <Input
                value={fieldLogicalName}
                onChange={(event) => setFieldLogicalName(event.target.value)}
                placeholder="logical_name"
                className="h-7 text-xs"
                required
              />
              <Input
                value={fieldDisplayName}
                onChange={(event) => setFieldDisplayName(event.target.value)}
                placeholder="Display Name"
                className="h-7 text-xs"
                required
              />
              <Select
                value={fieldType}
                onChange={(event) =>
                  setFieldType(
                    event.target.value as
                      | "text"
                      | "number"
                      | "boolean"
                      | "date"
                      | "datetime"
                      | "json"
                      | "relation",
                  )
                }
                className="h-7 text-xs"
              >
                <option value="text">Text</option>
                <option value="number">Number</option>
                <option value="boolean">Boolean</option>
                <option value="date">Date</option>
                <option value="datetime">DateTime</option>
                <option value="json">JSON</option>
                <option value="relation">Relation</option>
              </Select>
              {fieldType === "relation" ? (
                <Select
                  value={relationTarget}
                  onChange={(event) => setRelationTarget(event.target.value)}
                  className="h-7 text-xs"
                  required
                >
                  <option value="">Select relation target</option>
                  {allEntities
                    .filter((candidate) => candidate.logical_name !== entity.logicalName)
                    .map((candidate) => (
                      <option key={candidate.logical_name} value={candidate.logical_name}>
                        {candidate.display_name} ({candidate.logical_name})
                      </option>
                    ))}
                </Select>
              ) : null}
              <label className="inline-flex items-center gap-1 text-[11px] text-zinc-600">
                <Checkbox
                  checked={isRequired}
                  onChange={(event) => setIsRequired(event.target.checked)}
                />
                Required
              </label>
              <Button type="submit" size="sm" className="h-7 w-full" disabled={isSaving}>
                {isSaving ? "Creating..." : "Create Field"}
              </Button>
            </form>
          ) : null}

          {/* Forms */}
          <p className="px-2 pb-0.5 pt-1.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
            Forms
          </p>
          <div className="px-2 pb-1">
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-6 w-full text-[11px]"
              onClick={() => {
                if (!isCreateFormOpen && !formLogicalName.trim() && !formDisplayName.trim()) {
                  const suggested = suggestFormName(entity, formType);
                  setFormLogicalName(suggested.logicalName);
                  setFormDisplayName(suggested.displayName);
                }
                setIsCreateFormOpen((current) => !current);
              }}
            >
              {isCreateFormOpen ? "Close" : "+ Form"}
            </Button>
          </div>
          {isCreateFormOpen ? (
            <form
              className="mx-2 mb-1 space-y-1.5 rounded border border-zinc-200 bg-zinc-50 p-2"
              onSubmit={(event) => {
                event.preventDefault();
                void onCreateForm(entity.logicalName, {
                  logicalName: formLogicalName,
                  displayName: formDisplayName,
                  formType,
                }).then((createdLogicalName) => {
                  if (!createdLogicalName) return;
                  setFormLogicalName("");
                  setFormDisplayName("");
                  setFormType("main");
                  setIsCreateFormOpen(false);
                  onSelect({
                    kind: "form",
                    entityLogicalName: entity.logicalName,
                    formLogicalName: createdLogicalName,
                  });
                });
              }}
            >
              <Input
                value={formLogicalName}
                onChange={(event) => setFormLogicalName(event.target.value)}
                placeholder="form logical_name"
                className="h-7 text-xs"
                required
              />
              <Input
                value={formDisplayName}
                onChange={(event) => setFormDisplayName(event.target.value)}
                placeholder="Form Display Name"
                className="h-7 text-xs"
                required
              />
              <Select
                value={formType}
                onChange={(event) => {
                  const nextType = event.target.value as
                    | "main"
                    | "quick_create"
                    | "quick_view";
                  const currentSuggestion = suggestFormName(entity, formType);
                  const shouldReplaceSuggestion =
                    !formLogicalName.trim() ||
                    !formDisplayName.trim() ||
                    (formLogicalName === currentSuggestion.logicalName &&
                      formDisplayName === currentSuggestion.displayName);
                  setFormType(nextType);
                  if (shouldReplaceSuggestion) {
                    const nextSuggestion = suggestFormName(entity, nextType);
                    setFormLogicalName(nextSuggestion.logicalName);
                    setFormDisplayName(nextSuggestion.displayName);
                  }
                }}
                className="h-7 text-xs"
              >
                <option value="main">Main</option>
                <option value="quick_create">Quick Create</option>
                <option value="quick_view">Quick View</option>
              </Select>
              <Button type="submit" size="sm" className="h-7 w-full" disabled={isSaving}>
                {isSaving ? "Creating..." : "Create Form"}
              </Button>
            </form>
          ) : null}
          {entity.forms.length > 0
            ? entity.forms.map((form) => (
                <TreeButton
                  key={form.logicalName}
                  label={form.displayName}
                  sublabel={form.formType}
                  active={
                    selection.kind === "form" &&
                    selection.entityLogicalName === entity.logicalName &&
                    selection.formLogicalName === form.logicalName
                  }
                  onClick={() =>
                    onSelect({
                      kind: "form",
                      entityLogicalName: entity.logicalName,
                      formLogicalName: form.logicalName,
                    })
                  }
                  indent={1}
                />
              ))
            : (
                <p className="px-3 py-0.5 text-[11px] text-zinc-400">
                  {isLoading ? "Loading..." : "No forms"}
                </p>
              )}

          {/* Views */}
          <p className="px-2 pb-0.5 pt-1.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
            Views
          </p>
          <div className="px-2 pb-1">
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-6 w-full text-[11px]"
              onClick={() => {
                if (!isCreateViewOpen && !viewLogicalName.trim() && !viewDisplayName.trim()) {
                  const suggested = suggestViewName(entity, viewType);
                  setViewLogicalName(suggested.logicalName);
                  setViewDisplayName(suggested.displayName);
                }
                setIsCreateViewOpen((current) => !current);
              }}
              disabled={entityFields.length === 0}
            >
              {isCreateViewOpen ? "Close" : "+ View"}
            </Button>
          </div>
          {isCreateViewOpen ? (
            <form
              className="mx-2 mb-1 space-y-1.5 rounded border border-zinc-200 bg-zinc-50 p-2"
              onSubmit={(event) => {
                event.preventDefault();
                void onCreateView(entity.logicalName, {
                  logicalName: viewLogicalName,
                  displayName: viewDisplayName,
                  viewType,
                }).then((createdLogicalName) => {
                  if (!createdLogicalName) return;
                  setViewLogicalName("");
                  setViewDisplayName("");
                  setViewType("grid");
                  setIsCreateViewOpen(false);
                  onSelect({
                    kind: "view",
                    entityLogicalName: entity.logicalName,
                    viewLogicalName: createdLogicalName,
                  });
                });
              }}
            >
              <Input
                value={viewLogicalName}
                onChange={(event) => setViewLogicalName(event.target.value)}
                placeholder="view logical_name"
                className="h-7 text-xs"
                required
              />
              <Input
                value={viewDisplayName}
                onChange={(event) => setViewDisplayName(event.target.value)}
                placeholder="View Display Name"
                className="h-7 text-xs"
                required
              />
              <Select
                value={viewType}
                onChange={(event) => {
                  const nextType = event.target.value;
                  const currentSuggestion = suggestViewName(entity, viewType);
                  const shouldReplaceSuggestion =
                    !viewLogicalName.trim() ||
                    !viewDisplayName.trim() ||
                    (viewLogicalName === currentSuggestion.logicalName &&
                      viewDisplayName === currentSuggestion.displayName);
                  setViewType(nextType);
                  if (shouldReplaceSuggestion) {
                    const nextSuggestion = suggestViewName(entity, nextType);
                    setViewLogicalName(nextSuggestion.logicalName);
                    setViewDisplayName(nextSuggestion.displayName);
                  }
                }}
                className="h-7 text-xs"
              >
                <option value="grid">Grid</option>
                <option value="card">Card</option>
              </Select>
              <Button type="submit" size="sm" className="h-7 w-full" disabled={isSaving}>
                {isSaving ? "Creating..." : "Create View"}
              </Button>
            </form>
          ) : null}
          {entity.views.length > 0
            ? entity.views.map((view) => (
                <TreeButton
                  key={view.logicalName}
                  label={view.displayName}
                  sublabel={view.viewType}
                  active={
                    selection.kind === "view" &&
                    selection.entityLogicalName === entity.logicalName &&
                    selection.viewLogicalName === view.logicalName
                  }
                  onClick={() =>
                    onSelect({
                      kind: "view",
                      entityLogicalName: entity.logicalName,
                      viewLogicalName: view.logicalName,
                    })
                  }
                  indent={1}
                />
              ))
            : (
                <p className="px-3 py-0.5 text-[11px] text-zinc-400">
                  {isLoading ? "Loading..." : "No views"}
                </p>
              )}

          {/* Business Rules */}
          <p className="px-2 pb-0.5 pt-1.5 text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
            Business Rules
          </p>
          {entity.businessRules.length > 0
            ? entity.businessRules.map((rule) => (
                <TreeButton
                  key={rule.logicalName}
                  label={rule.displayName}
                  active={
                    selection.kind === "business-rule" &&
                    selection.entityLogicalName === entity.logicalName &&
                    selection.ruleLogicalName === rule.logicalName
                  }
                  onClick={() =>
                    onSelect({
                      kind: "business-rule",
                      entityLogicalName: entity.logicalName,
                      ruleLogicalName: rule.logicalName,
                    })
                  }
                  indent={1}
                />
              ))
            : (
                <p className="px-3 py-0.5 text-[11px] text-zinc-400">
                  {isLoading ? "Loading..." : "No rules"}
                </p>
              )}
        </div>
      ) : null}
    </div>
  );
}

function suggestFormName(
  entity: EntityTreeNode,
  formType: "main" | "quick_create" | "quick_view",
): { logicalName: string; displayName: string } {
  const existing = new Set(entity.forms.map((form) => form.logicalName));
  const baseLogicalName =
    formType === "main"
      ? "main_form"
      : formType === "quick_create"
        ? "quick_create"
        : "quick_view";
  const baseDisplayName =
    formType === "main"
      ? "Main Form"
      : formType === "quick_create"
        ? "Quick Create"
        : "Quick View";

  if (!existing.has(baseLogicalName)) {
    return { logicalName: baseLogicalName, displayName: baseDisplayName };
  }

  let index = 2;
  while (existing.has(`${baseLogicalName}_${index}`)) {
    index += 1;
  }

  return {
    logicalName: `${baseLogicalName}_${index}`,
    displayName: `${baseDisplayName} ${index}`,
  };
}

function suggestViewName(
  entity: EntityTreeNode,
  viewType: string,
): { logicalName: string; displayName: string } {
  const existing = new Set(entity.views.map((view) => view.logicalName));
  const baseLogicalName = viewType === "card" ? "card_view" : "main_view";
  const baseDisplayName = viewType === "card" ? "Card View" : "Main View";

  if (!existing.has(baseLogicalName)) {
    return { logicalName: baseLogicalName, displayName: baseDisplayName };
  }

  let index = 2;
  while (existing.has(`${baseLogicalName}_${index}`)) {
    index += 1;
  }

  return {
    logicalName: `${baseLogicalName}_${index}`,
    displayName: `${baseDisplayName} ${index}`,
  };
}

// ---------------------------------------------------------------------------
// Tree button primitive
// ---------------------------------------------------------------------------

type TreeButtonProps = {
  label: string;
  sublabel?: string;
  active: boolean;
  onClick: () => void;
  indent: number;
};

function TreeButton({ label, sublabel, active, onClick, indent }: TreeButtonProps) {
  return (
    <button
      type="button"
      className={cn(
        "flex w-full items-center gap-1.5 rounded-md px-2 py-1 text-left text-sm transition",
        indent > 0 && "pl-3",
        active
          ? "bg-zinc-900 font-medium text-white"
          : "text-zinc-700 hover:bg-zinc-100",
      )}
      onClick={onClick}
    >
      <span className="truncate">{label}</span>
      {sublabel ? (
        <span
          className={cn(
            "ml-auto text-[10px]",
            active ? "text-zinc-300" : "text-zinc-400",
          )}
        >
          {sublabel}
        </span>
      ) : null}
    </button>
  );
}
