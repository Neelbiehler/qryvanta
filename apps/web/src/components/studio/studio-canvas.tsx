"use client";

import { useState } from "react";

import { Button, Input, Label, Select, StatusBadge } from "@qryvanta/ui";

import type { StudioController } from "@/components/studio/hooks/use-studio-state";
import type { FormEditorState } from "@/components/studio/hooks/use-form-editor-state";
import {
  FormPreviewDataProvider,
} from "@/components/studio/form-builder/form-preview-data-provider";
import {
  InsertDropLine,
  SectionColumnDropZone,
  SectionGrid,
} from "@/components/studio/form-builder/section-grid";
import { SubgridPreview } from "@/components/studio/form-builder/subgrid-preview";
import { TabStrip } from "@/components/studio/form-builder/tab-strip";
import { WysiwygFieldRenderer } from "@/components/studio/form-builder/wysiwyg-field-renderer";
import { StudioPublishCanvas } from "@/components/studio/publish/studio-publish-canvas";
import { StudioSecurityCanvas } from "@/components/studio/security/studio-security-canvas";
import { StudioSitemapCanvas } from "@/components/studio/sitemap/studio-sitemap-canvas";
import { ViewCanvas } from "@/components/studio/view-builder/view-canvas";
import type { FieldResponse, PublishedSchemaResponse } from "@/lib/api";

type StudioCanvasProps = {
  studio: StudioController;
};

export function StudioCanvas({ studio }: StudioCanvasProps) {
  const { selection } = studio;

  switch (selection.kind) {
    case "overview":
      return <OverviewCanvas studio={studio} />;
    case "form":
      return studio.formEditor ? (
        <FormCanvas
          editor={studio.formEditor}
          publishedSchema={studio.getPublishedSchema(selection.entityLogicalName)}
          entityLogicalName={selection.entityLogicalName}
          formMeta={studio.formMeta}
          onSetFormMeta={studio.setFormMeta}
        />
      ) : (
        <EmptyCanvas message="Loading form..." />
      );
    case "view":
      return studio.viewEditor ? (
        <ViewCanvas
          appLogicalName={studio.selectedApp}
          entityLogicalName={selection.entityLogicalName}
          fields={studio.getPublishedSchema(selection.entityLogicalName)?.fields ?? []}
          viewEditor={studio.viewEditor}
          onRefreshPreviewRecords={async () => {
            await studio.refreshEntityPreviewRecords(selection.entityLogicalName);
          }}
        />
      ) : (
        <EmptyCanvas message="Loading view..." />
      );
    case "sitemap":
      return (
        <StudioSitemapCanvas
          appLogicalName={studio.selectedApp}
          entities={studio.entities}
        />
      );
    case "security":
      return (
        <StudioSecurityCanvas
          apps={studio.apps}
          entities={studio.entities}
          roles={studio.roles}
          selectedApp={studio.selectedApp}
          onChangeSelectedApp={studio.setSelectedApp}
        />
      );
    case "publish":
      return (
        <StudioPublishCanvas
          apps={studio.apps}
          entities={studio.entities}
          selectedApp={studio.selectedApp}
        />
      );
    case "business-rule":
      return <EmptyCanvas message="Business rule editor coming soon." />;
    default:
      return <EmptyCanvas message="Select an item from the tree." />;
  }
}

// ---------------------------------------------------------------------------
// Overview
// ---------------------------------------------------------------------------

function OverviewCanvas({ studio }: { studio: StudioController }) {
  return (
    <div className="flex h-full items-center justify-center rounded-xl border border-zinc-200 bg-white p-8">
      <div className="max-w-md space-y-4 text-center">
        <h2 className="font-serif text-2xl text-zinc-900">
          {studio.selectedAppDisplayName}
        </h2>
        <p className="text-sm text-zinc-600">
          Select a form or view from the entity tree on the left to start building.
        </p>
        <div className="flex flex-wrap justify-center gap-2">
          <StatusBadge tone="neutral">
            {studio.entities.length} entities
          </StatusBadge>
          <StatusBadge tone="neutral">
            {studio.roles.length} roles
          </StatusBadge>
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Empty placeholder
// ---------------------------------------------------------------------------

function EmptyCanvas({ message }: { message: string }) {
  return (
    <div className="flex h-full items-center justify-center rounded-xl border border-dashed border-zinc-300 bg-zinc-50 p-8">
      <p className="text-sm text-zinc-500">{message}</p>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Form Canvas — full structural form designer
// ---------------------------------------------------------------------------

type FormCanvasProps = {
  editor: FormEditorState;
  publishedSchema: PublishedSchemaResponse | null;
  entityLogicalName: string;
  formMeta: {
    logicalName: string;
    displayName: string;
    formType: string;
    headerFieldsText: string;
  } | null;
  onSetFormMeta: (patch: Partial<{ displayName: string; formType: string; headerFieldsText: string }>) => void;
};

function FormCanvas({
  editor,
  publishedSchema,
  entityLogicalName,
  formMeta,
  onSetFormMeta,
}: FormCanvasProps) {
  const [surfaceMode, setSurfaceMode] = useState<"design" | "preview">("design");
  const [viewport, setViewport] = useState<"desktop" | "tablet" | "mobile">("desktop");
  const [zoom, setZoom] = useState<"100" | "90" | "80">("100");

  const publishedFields = publishedSchema?.fields ?? [];
  const optionSets = publishedSchema?.option_sets ?? [];

  const viewportClassName =
    viewport === "desktop"
      ? "w-full"
      : viewport === "tablet"
        ? "mx-auto w-full max-w-3xl"
        : "mx-auto w-full max-w-sm";

  return (
    <div className="flex h-full min-h-0 flex-col rounded-xl border border-zinc-200 bg-white">
      {/* Form metadata bar */}
      {formMeta ? (
        <div className="flex flex-wrap items-end gap-3 border-b border-zinc-200 px-4 py-3">
          <div className="space-y-1">
            <Label htmlFor="studio_form_display_name" className="text-[11px]">Display Name</Label>
            <Input
              id="studio_form_display_name"
              value={formMeta.displayName}
              onChange={(event) => onSetFormMeta({ displayName: event.target.value })}
              className="h-8 w-48 text-sm"
            />
          </div>
          <div className="space-y-1">
            <Label htmlFor="studio_form_type" className="text-[11px]">Form Type</Label>
            <Select
              id="studio_form_type"
              value={formMeta.formType}
              onChange={(event) => onSetFormMeta({ formType: event.target.value })}
              className="h-8 w-32 text-sm"
            >
              <option value="main">Main</option>
              <option value="quick_create">Quick Create</option>
              <option value="quick_view">Quick View</option>
            </Select>
          </div>
          <div className="space-y-1">
            <Label htmlFor="studio_header_fields" className="text-[11px]">Header Fields</Label>
            <Input
              id="studio_header_fields"
              value={formMeta.headerFieldsText}
              onChange={(event) => onSetFormMeta({ headerFieldsText: event.target.value })}
              placeholder="name, status"
              className="h-8 w-48 text-sm"
            />
          </div>
          <div className="ml-auto flex items-center gap-2">
            <Select
              value={surfaceMode}
              onChange={(event) =>
                setSurfaceMode(event.target.value === "preview" ? "preview" : "design")
              }
              className="h-8 w-28 text-sm"
            >
              <option value="design">Design</option>
              <option value="preview">Preview</option>
            </Select>
            <Select
              value={viewport}
              onChange={(event) => {
                const value = event.target.value;
                setViewport(
                  value === "tablet" || value === "mobile" ? value : "desktop",
                );
              }}
              className="h-8 w-28 text-sm"
            >
              <option value="desktop">Desktop</option>
              <option value="tablet">Tablet</option>
              <option value="mobile">Mobile</option>
            </Select>
            <Select
              value={zoom}
              onChange={(event) => {
                const value = event.target.value;
                setZoom(value === "90" || value === "80" ? value : "100");
              }}
              className="h-8 w-24 text-sm"
            >
              <option value="100">100%</option>
              <option value="90">90%</option>
              <option value="80">80%</option>
            </Select>
            <Button type="button" variant="outline" size="sm" onClick={editor.addTab}>
              Add Tab
            </Button>
            <Button type="button" variant="outline" size="sm" onClick={editor.addSectionToActiveTab}>
              Add Section
            </Button>
          </div>
        </div>
      ) : null}

      {/* Tab bar */}
      <TabStrip
        tabs={editor.tabs}
        activeTabIndex={editor.activeTabIndex}
        onSelectTab={(index) => {
          editor.setActiveTabIndex(index);
          editor.setSelection({ kind: "tab", tabIndex: index });
        }}
        onReorderTabs={(sourceIndex, targetIndex) => {
          editor.updateTabs((current) => {
            const next = [...current];
            const [moved] = next.splice(sourceIndex, 1);
            next.splice(targetIndex, 0, moved);
            return next;
          });
          editor.setActiveTabIndex(targetIndex);
          editor.setSelection({ kind: "tab", tabIndex: targetIndex });
        }}
      />

      {/* Sections canvas */}
      <FormPreviewDataProvider
        entityLogicalName={entityLogicalName}
        enabled={surfaceMode === "preview"}
      >
        {({ values: previewValues, isLoading, errorMessage }) => (
          <div className="flex-1 overflow-y-auto p-4">
            <div
              className={`mx-auto origin-top rounded-lg border border-zinc-100 bg-zinc-50 p-3 transition ${viewportClassName}`}
              style={{ transform: `scale(${zoom === "100" ? "1" : zoom === "90" ? "0.9" : "0.8"})` }}
              role="application"
              aria-label="Form designer canvas"
              tabIndex={0}
              onKeyDown={editor.handleCanvasKeyDown}
            >
              {surfaceMode === "preview" && isLoading ? (
                <p className="mb-3 text-xs text-zinc-500">Loading preview record...</p>
              ) : null}
              {surfaceMode === "preview" && errorMessage ? (
                <p className="mb-3 text-xs text-amber-700">{errorMessage}</p>
              ) : null}

              <div className="space-y-4">
                {editor.activeTab.sections.map((section, sectionIndex) => (
                  <SectionBlock
                    key={`${editor.activeTab.logical_name}-${section.logical_name}`}
                    section={section}
                    sectionIndex={sectionIndex}
                    activeTabIndex={editor.activeTabIndex}
                    publishedFields={publishedFields}
                    optionSets={optionSets}
                    previewValues={previewValues}
                    previewMode={surfaceMode === "preview"}
                    placedFieldNames={editor.placedFieldNames}
                    activeDropLineId={editor.activeDropLineId}
                    onSetActiveDropLineId={editor.setActiveDropLineId}
                    onSelect={editor.setSelection}
                    onSetDragLabel={editor.setDragLabel}
                    onPlaceFieldInSection={editor.placeFieldInSection}
                    onDeleteField={editor.deleteField}
                    onAddFieldToSection={editor.addFieldToSection}
                  />
                ))}
              </div>
            </div>
          </div>
        )}
      </FormPreviewDataProvider>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Section block
// ---------------------------------------------------------------------------

type SectionBlockProps = {
  section: FormCanvasProps["editor"]["tabs"][number]["sections"][number];
  sectionIndex: number;
  activeTabIndex: number;
  publishedFields: FieldResponse[];
  optionSets: PublishedSchemaResponse["option_sets"];
  previewValues: Record<string, unknown>;
  previewMode: boolean;
  placedFieldNames: Set<string>;
  activeDropLineId: string | null;
  onSetActiveDropLineId: (lineId: string | null) => void;
  onSelect: FormEditorState["setSelection"];
  onSetDragLabel: (label: string | null) => void;
  onPlaceFieldInSection: FormEditorState["placeFieldInSection"];
  onDeleteField: FormEditorState["deleteField"];
  onAddFieldToSection: FormEditorState["addFieldToSection"];
};

function SectionBlock({
  section,
  sectionIndex,
  activeTabIndex,
  publishedFields,
  optionSets,
  previewValues,
  previewMode,
  placedFieldNames,
  activeDropLineId,
  onSetActiveDropLineId,
  onSelect,
  onSetDragLabel,
  onPlaceFieldInSection,
  onDeleteField,
  onAddFieldToSection,
}: SectionBlockProps) {
  return (
    <div
      className="rounded-lg border border-zinc-200 bg-zinc-50/50 p-3"
      role="button"
      tabIndex={0}
      onClick={() =>
        onSelect({ kind: "section", tabIndex: activeTabIndex, sectionIndex })
      }
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          onSelect({ kind: "section", tabIndex: activeTabIndex, sectionIndex });
        }
      }}
    >
      <div className="mb-2 flex items-center justify-between">
        <p className="text-sm font-semibold text-zinc-800">{section.display_name}</p>
        <div className="flex items-center gap-2">
          <StatusBadge tone="neutral">{section.columns} col</StatusBadge>
          {section.subgrids.length > 0 ? (
            <StatusBadge tone="neutral">{section.subgrids.length} subgrid</StatusBadge>
          ) : null}
        </div>
      </div>

      <SectionGrid columns={section.columns as 1 | 2 | 3}>
        {Array.from({ length: section.columns }).map((_, columnIndex) => {
          const fieldsInColumn = section.fields
            .filter((field) => field.column === columnIndex)
            .sort((left, right) => left.position - right.position);

          return (
            <ColumnZone
              key={`column-${columnIndex}`}
              columnIndex={columnIndex}
              sectionIndex={sectionIndex}
              activeTabIndex={activeTabIndex}
              fieldsInColumn={fieldsInColumn}
              allSectionFields={section.fields}
              publishedFields={publishedFields}
              optionSets={optionSets}
              previewValues={previewValues}
              previewMode={previewMode}
              placedFieldNames={placedFieldNames}
              activeDropLineId={activeDropLineId}
              onSetActiveDropLineId={onSetActiveDropLineId}
              onSelect={onSelect}
              onSetDragLabel={onSetDragLabel}
              onPlaceFieldInSection={onPlaceFieldInSection}
              onDeleteField={onDeleteField}
              onAddFieldToSection={onAddFieldToSection}
            />
          );
        })}
      </SectionGrid>

      {section.subgrids.length > 0 ? (
        <div className="mt-3 space-y-2">
          <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
            Sub-grids
          </p>
          {section.subgrids
            .slice()
            .sort((a, b) => a.position - b.position)
            .map((subgrid) => (
              <SubgridPreview key={subgrid.logical_name} subgrid={subgrid} />
            ))}
        </div>
      ) : null}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Column drop zone
// ---------------------------------------------------------------------------

type ColumnZoneProps = {
  columnIndex: number;
  sectionIndex: number;
  activeTabIndex: number;
  fieldsInColumn: FormCanvasProps["editor"]["tabs"][number]["sections"][number]["fields"];
  allSectionFields: FormCanvasProps["editor"]["tabs"][number]["sections"][number]["fields"];
  publishedFields: FieldResponse[];
  optionSets: PublishedSchemaResponse["option_sets"];
  previewValues: Record<string, unknown>;
  previewMode: boolean;
  placedFieldNames: Set<string>;
  activeDropLineId: string | null;
  onSetActiveDropLineId: (lineId: string | null) => void;
  onSelect: FormEditorState["setSelection"];
  onSetDragLabel: (label: string | null) => void;
  onPlaceFieldInSection: FormEditorState["placeFieldInSection"];
  onDeleteField: FormEditorState["deleteField"];
  onAddFieldToSection: FormEditorState["addFieldToSection"];
};

function ColumnZone({
  columnIndex,
  sectionIndex,
  activeTabIndex,
  fieldsInColumn,
  allSectionFields,
  publishedFields,
  optionSets,
  previewValues,
  previewMode,
  placedFieldNames,
  activeDropLineId,
  onSetActiveDropLineId,
  onSelect,
  onSetDragLabel,
  onPlaceFieldInSection,
  onDeleteField,
  onAddFieldToSection,
}: ColumnZoneProps) {
  return (
    <SectionColumnDropZone
      title={`Column ${columnIndex + 1}`}
      onDropField={(event) => {
        const fieldLogicalName = event.dataTransfer.getData("text/plain");
        const source =
          event.dataTransfer.getData("text/form-field-source") === "canvas"
            ? "canvas"
            : "palette";
        if (!fieldLogicalName) return;
        onPlaceFieldInSection(
          fieldLogicalName,
          activeTabIndex,
          sectionIndex,
          columnIndex,
          null,
          source,
        );
      }}
    >
        {fieldsInColumn.map((field) => {
          const fieldIndex = allSectionFields.findIndex(
            (c) =>
              c.field_logical_name === field.field_logical_name && c.position === field.position,
          );
          const metadata = publishedFields.find(
            (c) => c.logical_name === field.field_logical_name,
          );

          return (
            <div key={`${field.field_logical_name}-${field.position}`} className="space-y-0.5">
              <InsertDropLine
                lineId={`insert-${activeTabIndex}-${sectionIndex}-${columnIndex}-${field.position}`}
                activeLineId={activeDropLineId}
                onSetActiveLine={onSetActiveDropLineId}
                onDrop={(event) => {
                  const name = event.dataTransfer.getData("text/plain");
                  const src =
                    event.dataTransfer.getData("text/form-field-source") === "canvas"
                      ? "canvas"
                      : "palette";
                  if (!name) return;
                  onPlaceFieldInSection(
                    name,
                    activeTabIndex,
                    sectionIndex,
                    columnIndex,
                    fieldsInColumn.findIndex(
                      (c) =>
                        c.field_logical_name === field.field_logical_name &&
                        c.position === field.position,
                    ),
                    src,
                  );
                }}
              />
              <div
                draggable
                onDragStart={(event) => {
                  event.dataTransfer.setData("text/plain", field.field_logical_name);
                  event.dataTransfer.setData("text/form-field-source", "canvas");
                  event.dataTransfer.effectAllowed = "move";
                  onSetDragLabel(metadata?.display_name || field.field_logical_name);
                }}
                onDragEnd={() => onSetDragLabel(null)}
                className="w-full rounded-md border border-zinc-200 bg-zinc-50 px-2.5 py-2 text-left transition hover:border-zinc-300"
                onClick={(event) => {
                  event.stopPropagation();
                  onSelect({
                    kind: "field",
                    tabIndex: activeTabIndex,
                    sectionIndex,
                    fieldIndex,
                  });
                }}
                role="button"
                tabIndex={0}
              >
                <WysiwygFieldRenderer
                  placement={field}
                  field={metadata ?? null}
                  sampleValue={previewValues[field.field_logical_name]}
                  optionSets={optionSets}
                  previewMode={previewMode}
                />
                <div className="mt-1 flex items-center justify-between">
                  <p className="font-mono text-[10px] text-zinc-500">{field.field_logical_name}</p>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={(event) => {
                      event.stopPropagation();
                      onDeleteField(activeTabIndex, sectionIndex, fieldIndex);
                    }}
                    className="h-6 px-1.5 text-xs text-zinc-500 hover:text-red-600"
                  >
                    Remove
                  </Button>
                </div>
              </div>
            </div>
          );
        })}

        <InsertDropLine
          lineId={`insert-${activeTabIndex}-${sectionIndex}-${columnIndex}-end`}
          activeLineId={activeDropLineId}
          onSetActiveLine={onSetActiveDropLineId}
          onDrop={(event) => {
            const name = event.dataTransfer.getData("text/plain");
            const src =
              event.dataTransfer.getData("text/form-field-source") === "canvas"
                ? "canvas"
                : "palette";
            if (!name) return;
            onPlaceFieldInSection(
              name,
              activeTabIndex,
              sectionIndex,
              columnIndex,
              fieldsInColumn.length,
              src,
            );
          }}
        />

        <Select
          value=""
          onChange={(event) => {
            const name = event.target.value;
            if (!name) return;
            onAddFieldToSection(name, activeTabIndex, sectionIndex, columnIndex);
          }}
          className="h-7 text-xs"
        >
          <option value="">Quick add field...</option>
          {publishedFields
            .filter((f) => !placedFieldNames.has(f.logical_name))
            .map((f) => (
              <option key={f.logical_name} value={f.logical_name}>
                {f.display_name}
              </option>
            ))}
        </Select>
    </SectionColumnDropZone>
  );
}
