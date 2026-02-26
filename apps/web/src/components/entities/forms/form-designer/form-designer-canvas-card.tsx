import { type DragEvent, type KeyboardEvent } from "react";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Select,
  StatusBadge,
} from "@qryvanta/ui";

import type {
  FormTab,
  SelectionState,
} from "@/components/entities/forms/form-designer/types";
import type { PublishedSchemaResponse } from "@/lib/api";

type FormDesignerCanvasCardProps = {
  tabs: FormTab[];
  activeTab: FormTab;
  activeTabIndex: number;
  activeDropLineId: string | null;
  publishedFields: PublishedSchemaResponse["fields"];
  placedFieldNames: Set<string>;
  isPreviewMode: boolean;
  onSetActiveDropLineId: (lineId: string | null) => void;
  onSetActiveTabIndex: (tabIndex: number) => void;
  onSelect: (selection: SelectionState) => void;
  onSetDragLabel: (label: string | null) => void;
  onUpdateTabs: (mutator: (current: FormTab[]) => FormTab[]) => void;
  onAddTab: () => void;
  onAddSectionToActiveTab: () => void;
  onCanvasKeyDown: (event: KeyboardEvent<HTMLDivElement>) => void;
  onPlaceFieldInSection: (
    fieldLogicalName: string,
    tabIndex: number,
    sectionIndex: number,
    column: number,
    insertAt: number | null,
    source: "palette" | "canvas",
  ) => void;
  onDeleteField: (tabIndex: number, sectionIndex: number, fieldIndex: number) => void;
  onAddFieldToSection: (
    fieldLogicalName: string,
    tabIndex: number,
    sectionIndex: number,
    column: number,
  ) => void;
};

export function FormDesignerCanvasCard({
  tabs,
  activeTab,
  activeTabIndex,
  activeDropLineId,
  publishedFields,
  placedFieldNames,
  isPreviewMode,
  onSetActiveDropLineId,
  onSetActiveTabIndex,
  onSelect,
  onSetDragLabel,
  onUpdateTabs,
  onAddTab,
  onAddSectionToActiveTab,
  onCanvasKeyDown,
  onPlaceFieldInSection,
  onDeleteField,
  onAddFieldToSection,
}: FormDesignerCanvasCardProps) {
  return (
    <Card>
      <CardHeader className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <div>
          <CardTitle className="text-base">Canvas</CardTitle>
          <CardDescription>
            Tabs, sections, and columns are rendered as worker-surface layout blocks.
          </CardDescription>
        </div>
        <div className="flex gap-2">
          <Button type="button" variant="outline" onClick={onAddTab}>
            Add Tab
          </Button>
          <Button type="button" variant="outline" onClick={onAddSectionToActiveTab}>
            Add Section
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap gap-2">
          {tabs.map((tab, index) => (
            <Button
              key={tab.logical_name}
              type="button"
              variant={index === activeTabIndex ? "default" : "outline"}
              onClick={() => {
                onSetActiveTabIndex(index);
                onSelect({ kind: "tab", tabIndex: index });
              }}
              draggable
              onDragStart={(event) => {
                event.dataTransfer.setData("text/tab-index", String(index));
              }}
              onDragOver={(event) => event.preventDefault()}
              onDrop={(event) => {
                const sourceIndex = Number.parseInt(event.dataTransfer.getData("text/tab-index"), 10);
                if (Number.isNaN(sourceIndex) || sourceIndex === index) {
                  return;
                }
                onUpdateTabs((current) => {
                  const next = [...current];
                  const [moved] = next.splice(sourceIndex, 1);
                  next.splice(index, 0, moved);
                  return next;
                });
                onSetActiveTabIndex(index);
                onSelect({ kind: "tab", tabIndex: index });
              }}
            >
              {tab.display_name}
            </Button>
          ))}
        </div>

        <div
          className="space-y-3"
          role="application"
          aria-label="Form designer canvas"
          tabIndex={0}
          onKeyDown={onCanvasKeyDown}
        >
          {activeTab.sections.map((section, sectionIndex) => (
            <div
              key={`${activeTab.logical_name}-${section.logical_name}`}
              className="rounded-md border border-zinc-200 bg-zinc-50 p-3"
              role="button"
              tabIndex={0}
              onClick={() =>
                onSelect({
                  kind: "section",
                  tabIndex: activeTabIndex,
                  sectionIndex,
                })
              }
              onKeyDown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  onSelect({
                    kind: "section",
                    tabIndex: activeTabIndex,
                    sectionIndex,
                  });
                }
              }}
            >
              <div className="mb-2 flex items-center justify-between">
                <p className="text-sm font-semibold text-zinc-800">{section.display_name}</p>
                <div className="flex items-center gap-2">
                  <StatusBadge tone="neutral">Columns {section.columns}</StatusBadge>
                  <StatusBadge tone="neutral">Sub-grids {section.subgrids.length}</StatusBadge>
                </div>
              </div>
              <div
                className={
                  section.columns === 1
                    ? "grid gap-2"
                    : section.columns === 2
                      ? "grid gap-2 md:grid-cols-2"
                      : "grid gap-2 md:grid-cols-3"
                }
              >
                {Array.from({ length: section.columns }).map((_, columnIndex) => {
                  const fieldsInColumn = section.fields
                    .filter((field) => field.column === columnIndex)
                    .sort((left, right) => left.position - right.position);

                  return (
                    <div
                      key={`column-${columnIndex}`}
                      className="min-h-24 rounded-md border border-dashed border-zinc-300 bg-white p-2"
                      onDragOver={(event) => event.preventDefault()}
                      onDrop={(event) => {
                        event.preventDefault();
                        const fieldLogicalName = event.dataTransfer.getData("text/plain");
                        const source =
                          event.dataTransfer.getData("text/form-field-source") === "canvas"
                            ? "canvas"
                            : "palette";
                        if (!fieldLogicalName) {
                          return;
                        }
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
                      <p className="mb-2 text-[11px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
                        Column {columnIndex + 1}
                      </p>
                      <div className="space-y-2">
                        {fieldsInColumn.map((field) => {
                          const fieldIndex = section.fields.findIndex(
                            (candidate) =>
                              candidate.field_logical_name === field.field_logical_name &&
                              candidate.position === field.position,
                          );
                          const metadata = publishedFields.find(
                            (candidate) => candidate.logical_name === field.field_logical_name,
                          );

                          return (
                            <div key={`${field.field_logical_name}-${field.position}`} className="space-y-1">
                              <ColumnDropLine
                                lineId={`field-insert-${activeTabIndex}-${sectionIndex}-${columnIndex}-${field.position}`}
                                activeLineId={activeDropLineId}
                                onSetActiveLine={onSetActiveDropLineId}
                                onDrop={(event) => {
                                  const fieldLogicalName = event.dataTransfer.getData("text/plain");
                                  const source =
                                    event.dataTransfer.getData("text/form-field-source") === "canvas"
                                      ? "canvas"
                                      : "palette";
                                  if (!fieldLogicalName) {
                                    return;
                                  }
                                  onPlaceFieldInSection(
                                    fieldLogicalName,
                                    activeTabIndex,
                                    sectionIndex,
                                    columnIndex,
                                    fieldsInColumn.findIndex(
                                      (candidate) =>
                                        candidate.field_logical_name === field.field_logical_name &&
                                        candidate.position === field.position,
                                    ),
                                    source,
                                  );
                                }}
                              />
                              <button
                                type="button"
                                draggable
                                onDragStart={(event) => {
                                  event.dataTransfer.setData("text/plain", field.field_logical_name);
                                  event.dataTransfer.setData("text/form-field-source", "canvas");
                                  event.dataTransfer.effectAllowed = "move";
                                  onSetDragLabel(metadata?.display_name || field.field_logical_name);
                                }}
                                onDragEnd={() => onSetDragLabel(null)}
                                className="w-full rounded-md border border-zinc-200 bg-zinc-50 px-2 py-2 text-left"
                                onClick={(event) => {
                                  event.stopPropagation();
                                  onSelect({
                                    kind: "field",
                                    tabIndex: activeTabIndex,
                                    sectionIndex,
                                    fieldIndex,
                                  });
                                }}
                              >
                                <p className="text-sm font-medium text-zinc-800">
                                  {field.label_override?.trim() || metadata?.display_name || field.field_logical_name}
                                </p>
                                <p className="font-mono text-xs text-zinc-500">
                                  {field.field_logical_name}
                                </p>
                                {!isPreviewMode ? (
                                  <Button
                                    type="button"
                                    variant="ghost"
                                    size="sm"
                                    onClick={(event) => {
                                      event.stopPropagation();
                                      onDeleteField(activeTabIndex, sectionIndex, fieldIndex);
                                    }}
                                    className="mt-1"
                                  >
                                    Remove
                                  </Button>
                                ) : null}
                              </button>
                            </div>
                          );
                        })}

                        <ColumnDropLine
                          lineId={`field-insert-${activeTabIndex}-${sectionIndex}-${columnIndex}-end`}
                          activeLineId={activeDropLineId}
                          onSetActiveLine={onSetActiveDropLineId}
                          onDrop={(event) => {
                            const fieldLogicalName = event.dataTransfer.getData("text/plain");
                            const source =
                              event.dataTransfer.getData("text/form-field-source") === "canvas"
                                ? "canvas"
                                : "palette";
                            if (!fieldLogicalName) {
                              return;
                            }
                            onPlaceFieldInSection(
                              fieldLogicalName,
                              activeTabIndex,
                              sectionIndex,
                              columnIndex,
                              fieldsInColumn.length,
                              source,
                            );
                          }}
                        />

                        {!isPreviewMode ? (
                          <Select
                            value=""
                            onChange={(event) => {
                              const fieldLogicalName = event.target.value;
                              if (!fieldLogicalName) {
                                return;
                              }
                              onAddFieldToSection(
                                fieldLogicalName,
                                activeTabIndex,
                                sectionIndex,
                                columnIndex,
                              );
                            }}
                          >
                            <option value="">Quick add field...</option>
                            {publishedFields
                              .filter((field) => !placedFieldNames.has(field.logical_name))
                              .map((field) => (
                                <option key={field.logical_name} value={field.logical_name}>
                                  {field.display_name}
                                </option>
                              ))}
                          </Select>
                        ) : null}
                      </div>
                    </div>
                  );
                })}
              </div>
              {section.subgrids.length > 0 ? (
                <div className="mt-3 space-y-1 rounded-md border border-dashed border-zinc-300 bg-zinc-100 p-2">
                  <p className="text-[11px] font-semibold uppercase tracking-[0.14em] text-zinc-500">
                    Sub-grids
                  </p>
                  {section.subgrids
                    .slice()
                    .sort((left, right) => left.position - right.position)
                    .map((subgrid) => (
                      <p key={subgrid.logical_name} className="text-xs text-zinc-700">
                        {subgrid.display_name} ({subgrid.target_entity_logical_name || "target"})
                      </p>
                    ))}
                </div>
              ) : null}
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}

type ColumnDropLineProps = {
  lineId: string;
  activeLineId: string | null;
  onSetActiveLine: (lineId: string | null) => void;
  label?: string;
  onDrop: (event: DragEvent<HTMLDivElement>) => void;
};

function ColumnDropLine({
  lineId,
  activeLineId,
  onSetActiveLine,
  label,
  onDrop,
}: ColumnDropLineProps) {
  const isActive = activeLineId === lineId;
  return (
    <div
      className={`rounded border border-dashed px-2 py-0.5 text-[10px] transition ${isActive ? "border-emerald-400 bg-emerald-100 text-emerald-900" : "border-transparent text-transparent hover:border-emerald-300 hover:bg-emerald-100 hover:text-emerald-800"}`}
      onDragOver={(event) => {
        event.preventDefault();
        onSetActiveLine(lineId);
      }}
      onDragEnter={() => onSetActiveLine(lineId)}
      onDragLeave={() => onSetActiveLine(null)}
      onDrop={(event) => {
        event.preventDefault();
        onSetActiveLine(null);
        onDrop(event);
      }}
      aria-hidden
    >
      {label ?? "Insert here"}
    </div>
  );
}
