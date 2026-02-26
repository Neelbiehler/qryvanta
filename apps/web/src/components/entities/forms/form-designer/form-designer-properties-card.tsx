import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Checkbox,
  Input,
  Label,
  Select,
} from "@qryvanta/ui";

import type {
  FormFieldPlacement,
  FormSection,
  FormSubgrid,
  FormTab,
  SelectionState,
} from "@/components/entities/forms/form-designer/types";

type FormDesignerPropertiesCardProps = {
  selection: SelectionState;
  selectedTab: FormTab | null;
  selectedSection: FormSection | null;
  selectedField: FormFieldPlacement | null;
  onUpdateSelectedTab: (patch: Partial<FormTab>) => void;
  onUpdateSelectedSection: (patch: Partial<FormSection>) => void;
  onUpdateSelectedField: (patch: Partial<FormFieldPlacement>) => void;
  onAddSubgridToSelectedSection: () => void;
  onUpdateSubgridInSelectedSection: (
    subgridIndex: number,
    patch: Partial<FormSubgrid>,
  ) => void;
  onRemoveSubgridFromSelectedSection: (subgridIndex: number) => void;
};

export function FormDesignerPropertiesCard({
  selection,
  selectedTab,
  selectedSection,
  selectedField,
  onUpdateSelectedTab,
  onUpdateSelectedSection,
  onUpdateSelectedField,
  onAddSubgridToSelectedSection,
  onUpdateSubgridInSelectedSection,
  onRemoveSubgridFromSelectedSection,
}: FormDesignerPropertiesCardProps) {
  return (
    <Card className="h-fit">
      <CardHeader>
        <CardTitle className="text-base">Properties</CardTitle>
        <CardDescription>Edit selected tab, section, or field behavior.</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {selection.kind === "tab" && selectedTab ? (
          <div className="space-y-3">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Tab
            </p>
            <div className="space-y-2">
              <Label htmlFor="selected_tab_display_name">Display Name</Label>
              <Input
                id="selected_tab_display_name"
                value={selectedTab.display_name}
                onChange={(event) => onUpdateSelectedTab({ display_name: event.target.value })}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="selected_tab_logical_name">Logical Name</Label>
              <Input
                id="selected_tab_logical_name"
                value={selectedTab.logical_name}
                onChange={(event) => onUpdateSelectedTab({ logical_name: event.target.value })}
              />
            </div>
            <div className="flex items-center gap-2">
              <Checkbox
                id="selected_tab_visible"
                checked={selectedTab.visible}
                onChange={(event) => onUpdateSelectedTab({ visible: event.target.checked })}
              />
              <Label htmlFor="selected_tab_visible">Visible</Label>
            </div>
          </div>
        ) : null}

        {(selection.kind === "section" || selection.kind === "field") && selectedSection ? (
          <div className="space-y-3">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Section
            </p>
            <div className="space-y-2">
              <Label htmlFor="selected_section_display_name">Display Name</Label>
              <Input
                id="selected_section_display_name"
                value={selectedSection.display_name}
                onChange={(event) =>
                  onUpdateSelectedSection({ display_name: event.target.value })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="selected_section_columns">Columns</Label>
              <Select
                id="selected_section_columns"
                value={String(selectedSection.columns)}
                onChange={(event) =>
                  onUpdateSelectedSection({
                    columns: Number.parseInt(event.target.value, 10) as 1 | 2 | 3,
                  })
                }
              >
                <option value="1">1</option>
                <option value="2">2</option>
                <option value="3">3</option>
              </Select>
            </div>
            <div className="flex items-center gap-2">
              <Checkbox
                id="selected_section_visible"
                checked={selectedSection.visible}
                onChange={(event) => onUpdateSelectedSection({ visible: event.target.checked })}
              />
              <Label htmlFor="selected_section_visible">Visible</Label>
            </div>

            <div className="space-y-2 border-t border-zinc-200 pt-3">
              <div className="flex items-center justify-between">
                <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
                  Sub-grids
                </p>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  onClick={onAddSubgridToSelectedSection}
                >
                  Add Sub-grid
                </Button>
              </div>

              {selectedSection.subgrids.length > 0 ? (
                selectedSection.subgrids
                  .slice()
                  .sort((left, right) => left.position - right.position)
                  .map((subgrid, subgridIndex) => (
                    <div
                      key={`${subgrid.logical_name}-${subgridIndex}`}
                      className="space-y-2 rounded-md border border-zinc-200 p-2"
                    >
                      <Input
                        value={subgrid.display_name}
                        onChange={(event) =>
                          onUpdateSubgridInSelectedSection(subgridIndex, {
                            display_name: event.target.value,
                          })
                        }
                        placeholder="Display name"
                      />
                      <Input
                        value={subgrid.logical_name}
                        onChange={(event) =>
                          onUpdateSubgridInSelectedSection(subgridIndex, {
                            logical_name: event.target.value,
                          })
                        }
                        placeholder="Logical name"
                      />
                      <Input
                        value={subgrid.target_entity_logical_name}
                        onChange={(event) =>
                          onUpdateSubgridInSelectedSection(subgridIndex, {
                            target_entity_logical_name: event.target.value,
                          })
                        }
                        placeholder="Target entity logical name"
                      />
                      <Input
                        value={subgrid.relation_field_logical_name}
                        onChange={(event) =>
                          onUpdateSubgridInSelectedSection(subgridIndex, {
                            relation_field_logical_name: event.target.value,
                          })
                        }
                        placeholder="Target relation field logical name"
                      />
                      <Input
                        value={subgrid.columns.join(", ")}
                        onChange={(event) =>
                          onUpdateSubgridInSelectedSection(subgridIndex, {
                            columns: event.target.value
                              .split(",")
                              .map((value) => value.trim())
                              .filter((value, index, values) =>
                                value.length > 0 && values.indexOf(value) === index,
                              ),
                          })
                        }
                        placeholder="Columns (comma-separated, optional)"
                      />
                      <Button
                        type="button"
                        size="sm"
                        variant="ghost"
                        onClick={() => onRemoveSubgridFromSelectedSection(subgridIndex)}
                      >
                        Remove Sub-grid
                      </Button>
                    </div>
                  ))
              ) : (
                <p className="text-xs text-zinc-500">No sub-grids in this section.</p>
              )}
            </div>
          </div>
        ) : null}

        {selection.kind === "field" && selectedField ? (
          <div className="space-y-3 border-t border-zinc-200 pt-3">
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Field
            </p>
            <div className="space-y-2">
              <Label htmlFor="selected_field_label_override">Label Override</Label>
              <Input
                id="selected_field_label_override"
                value={selectedField.label_override ?? ""}
                onChange={(event) =>
                  onUpdateSelectedField({
                    label_override: event.target.value.trim() || null,
                  })
                }
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="selected_field_required_override">Required Override</Label>
              <Select
                id="selected_field_required_override"
                value={
                  selectedField.required_override === null
                    ? "inherit"
                    : selectedField.required_override
                      ? "required"
                      : "optional"
                }
                onChange={(event) => {
                  const value = event.target.value;
                  onUpdateSelectedField({
                    required_override: value === "inherit" ? null : value === "required",
                  });
                }}
              >
                <option value="inherit">Inherit</option>
                <option value="required">Required</option>
                <option value="optional">Optional</option>
              </Select>
            </div>
            <div className="grid gap-2 md:grid-cols-2">
              <div className="flex items-center gap-2">
                <Checkbox
                  id="selected_field_visible"
                  checked={selectedField.visible}
                  onChange={(event) => onUpdateSelectedField({ visible: event.target.checked })}
                />
                <Label htmlFor="selected_field_visible">Visible</Label>
              </div>
              <div className="flex items-center gap-2">
                <Checkbox
                  id="selected_field_read_only"
                  checked={selectedField.read_only}
                  onChange={(event) =>
                    onUpdateSelectedField({ read_only: event.target.checked })
                  }
                />
                <Label htmlFor="selected_field_read_only">Read Only</Label>
              </div>
            </div>
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
