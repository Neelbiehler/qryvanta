"use client";

import {
  Button,
  Checkbox,
  Input,
  Label,
  Select,
} from "@qryvanta/ui";

import type { StudioController } from "@/components/studio/hooks/use-studio-state";
import type { FormEditorState } from "@/components/studio/hooks/use-form-editor-state";
import { ViewPropertiesPanel } from "@/components/studio/view-builder/view-properties-panel";
import type { FieldResponse } from "@/lib/api";

type StudioPropertiesPanelProps = {
  studio: StudioController;
};

export function StudioPropertiesPanel({ studio }: StudioPropertiesPanelProps) {
  if (studio.selection.kind === "form" && studio.formEditor) {
    const publishedFields =
      studio.getPublishedSchema(studio.selection.entityLogicalName)?.fields ?? [];
    return (
      <FormPropertiesPanel editor={studio.formEditor} publishedFields={publishedFields} />
    );
  }

  if (studio.selection.kind === "view" && studio.viewEditor) {
    const fields =
      studio.getPublishedSchema(studio.selection.entityLogicalName)?.fields ?? [];
    return (
      <ViewPropertiesPanel
        viewEditor={studio.viewEditor}
        fields={fields}
        viewMeta={studio.viewMeta}
        onSetViewMeta={studio.setViewMeta}
      />
    );
  }

  return (
    <aside className="rounded-xl border border-zinc-200 bg-white p-3">
      <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
        Properties
      </p>
      <p className="mt-2 text-xs text-zinc-500">
        Select a form element to edit its properties.
      </p>
    </aside>
  );
}

// ---------------------------------------------------------------------------
// Form properties — shows tab / section / field properties
// ---------------------------------------------------------------------------

function FormPropertiesPanel({
  editor,
  publishedFields,
}: {
  editor: FormEditorState;
  publishedFields: FieldResponse[];
}) {
  const { selection, selectedTab, selectedSection, selectedField } = editor;

  return (
    <aside className="flex h-full min-h-0 flex-col overflow-y-auto rounded-xl border border-zinc-200 bg-white">
      <div className="border-b border-zinc-200 px-3 py-3">
        <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
          Properties
        </p>
        <p className="mt-0.5 text-[11px] text-zinc-500">
          Edit selected tab, section, or field.
        </p>
      </div>

      <div className="flex-1 space-y-4 overflow-y-auto p-3">
        {/* Field palette (draggable) */}
        <div>
          <p className="mb-2 text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
            Field Palette
          </p>
          <div className="max-h-48 space-y-1 overflow-y-auto">
            {publishedFields.map((field) => (
              <button
                key={field.logical_name}
                type="button"
                draggable
                onDragStart={(event) => {
                  event.dataTransfer.setData("text/plain", field.logical_name);
                  event.dataTransfer.setData("text/form-field-source", "palette");
                  editor.setDragLabel(field.display_name || field.logical_name);
                }}
                onDragEnd={() => editor.setDragLabel(null)}
                className="flex w-full items-center gap-1.5 rounded-md border border-zinc-200 bg-white px-2 py-1.5 text-left text-xs hover:border-emerald-300"
              >
                <span className="truncate font-medium text-zinc-800">{field.display_name}</span>
                <span className="ml-auto text-[10px] text-zinc-400">{field.field_type}</span>
                {editor.placedFieldNames.has(field.logical_name) ? (
                  <span className="text-[10px] text-emerald-600">*</span>
                ) : null}
              </button>
            ))}
            {publishedFields.length === 0 ? (
              <p className="text-[11px] text-zinc-400">Publish entity to see fields.</p>
            ) : null}
          </div>
        </div>

        {/* Tab properties */}
        {selection.kind === "tab" && selectedTab ? (
          <div className="space-y-3 border-t border-zinc-200 pt-3">
            <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
              Tab
            </p>
            <div className="space-y-1.5">
              <Label htmlFor="" className="text-[11px]">Display Name</Label>
              <Input
                value={selectedTab.display_name}
                onChange={(e) => editor.updateSelectedTab({ display_name: e.target.value })}
                className="h-8 text-sm"
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="" className="text-[11px]">Logical Name</Label>
              <Input
                value={selectedTab.logical_name}
                onChange={(e) => editor.updateSelectedTab({ logical_name: e.target.value })}
                className="h-8 text-sm"
              />
            </div>
            <div className="flex items-center gap-2">
              <Checkbox
                id="tab_visible"
                checked={selectedTab.visible}
                onChange={(e) => editor.updateSelectedTab({ visible: e.target.checked })}
              />
              <Label htmlFor="tab_visible" className="text-xs">
                Visible
              </Label>
            </div>
          </div>
        ) : null}

        {/* Section properties */}
        {(selection.kind === "section" || selection.kind === "field") && selectedSection ? (
          <div className="space-y-3 border-t border-zinc-200 pt-3">
            <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
              Section
            </p>
            <div className="space-y-1.5">
              <Label htmlFor="" className="text-[11px]">Display Name</Label>
              <Input
                value={selectedSection.display_name}
                onChange={(e) =>
                  editor.updateSelectedSection({ display_name: e.target.value })
                }
                className="h-8 text-sm"
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="" className="text-[11px]">Columns</Label>
              <Select
                value={String(selectedSection.columns)}
                onChange={(e) =>
                  editor.updateSelectedSection({
                    columns: Number.parseInt(e.target.value, 10) as 1 | 2 | 3,
                  })
                }
                className="h-8 text-sm"
              >
                <option value="1">1</option>
                <option value="2">2</option>
                <option value="3">3</option>
              </Select>
            </div>
            <div className="flex items-center gap-2">
              <Checkbox
                id="section_visible"
                checked={selectedSection.visible}
                onChange={(e) =>
                  editor.updateSelectedSection({ visible: e.target.checked })
                }
              />
              <Label htmlFor="section_visible" className="text-xs">
                Visible
              </Label>
            </div>

            {/* Sub-grids */}
            <div className="space-y-2 border-t border-zinc-200 pt-3">
              <div className="flex items-center justify-between">
                <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
                  Sub-grids
                </p>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={editor.addSubgridToSelectedSection}
                  className="h-6 text-[11px]"
                >
                  Add
                </Button>
              </div>
              {selectedSection.subgrids.length > 0
                ? selectedSection.subgrids
                    .slice()
                    .sort((a, b) => a.position - b.position)
                    .map((subgrid, subgridIndex) => (
                      <div
                        key={`${subgrid.logical_name}-${subgridIndex}`}
                        className="space-y-1.5 rounded-md border border-zinc-200 p-2"
                      >
                        <Input
                          value={subgrid.display_name}
                          onChange={(e) =>
                            editor.updateSubgridInSelectedSection(subgridIndex, {
                              display_name: e.target.value,
                            })
                          }
                          placeholder="Display name"
                          className="h-7 text-xs"
                        />
                        <Input
                          value={subgrid.target_entity_logical_name}
                          onChange={(e) =>
                            editor.updateSubgridInSelectedSection(subgridIndex, {
                              target_entity_logical_name: e.target.value,
                            })
                          }
                          placeholder="Target entity"
                          className="h-7 text-xs"
                        />
                        <Input
                          value={subgrid.relation_field_logical_name}
                          onChange={(e) =>
                            editor.updateSubgridInSelectedSection(subgridIndex, {
                              relation_field_logical_name: e.target.value,
                            })
                          }
                          placeholder="Relation field"
                          className="h-7 text-xs"
                        />
                        <Button
                          type="button"
                          size="sm"
                          variant="ghost"
                          onClick={() =>
                            editor.removeSubgridFromSelectedSection(subgridIndex)
                          }
                          className="h-6 text-[11px] text-zinc-500 hover:text-red-600"
                        >
                          Remove
                        </Button>
                      </div>
                    ))
                : (
                    <p className="text-[11px] text-zinc-400">No sub-grids.</p>
                  )}
            </div>
          </div>
        ) : null}

        {/* Field properties */}
        {selection.kind === "field" && selectedField ? (
          <div className="space-y-3 border-t border-zinc-200 pt-3">
            <p className="text-[10px] font-semibold uppercase tracking-[0.14em] text-zinc-400">
              Field
            </p>
            <div className="space-y-1.5">
              <Label htmlFor="" className="text-[11px]">Label Override</Label>
              <Input
                value={selectedField.label_override ?? ""}
                onChange={(e) =>
                  editor.updateSelectedField({
                    label_override: e.target.value.trim() || null,
                  })
                }
                className="h-8 text-sm"
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="" className="text-[11px]">Required Override</Label>
              <Select
                value={
                  selectedField.required_override === null
                    ? "inherit"
                    : selectedField.required_override
                      ? "required"
                      : "optional"
                }
                onChange={(e) => {
                  const value = e.target.value;
                  editor.updateSelectedField({
                    required_override: value === "inherit" ? null : value === "required",
                  });
                }}
                className="h-8 text-sm"
              >
                <option value="inherit">Inherit</option>
                <option value="required">Required</option>
                <option value="optional">Optional</option>
              </Select>
            </div>
            <div className="grid grid-cols-2 gap-2">
              <div className="flex items-center gap-2">
                <Checkbox
                  id="field_visible"
                  checked={selectedField.visible}
                  onChange={(e) =>
                    editor.updateSelectedField({ visible: e.target.checked })
                  }
                />
                <Label htmlFor="field_visible" className="text-xs">
                  Visible
                </Label>
              </div>
              <div className="flex items-center gap-2">
                <Checkbox
                  id="field_read_only"
                  checked={selectedField.read_only}
                  onChange={(e) =>
                    editor.updateSelectedField({ read_only: e.target.checked })
                  }
                />
                <Label htmlFor="field_read_only" className="text-xs">
                  Read Only
                </Label>
              </div>
            </div>
          </div>
        ) : null}
      </div>
    </aside>
  );
}
