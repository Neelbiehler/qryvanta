"use client";

import { type DragEvent, type KeyboardEvent, useEffect, useMemo, useState } from "react";
import { useRouter } from "next/navigation";

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
  Notice,
  Select,
  StatusBadge,
} from "@qryvanta/ui";

import {
  apiFetch,
  type CreateFormRequest,
  type FormResponse,
  type PublishedSchemaResponse,
} from "@/lib/api";

type FormDesignerPanelProps = {
  entityLogicalName: string;
  initialForm: FormResponse | null;
  initialForms: FormResponse[];
  publishedSchema: PublishedSchemaResponse | null;
};

type FormTypeValue = "main" | "quick_create" | "quick_view";

type FormFieldPlacement = {
  field_logical_name: string;
  column: number;
  position: number;
  visible: boolean;
  read_only: boolean;
  required_override: boolean | null;
  label_override: string | null;
};

type FormSubgrid = {
  logical_name: string;
  display_name: string;
  target_entity_logical_name: string;
  relation_field_logical_name: string;
  position: number;
  columns: string[];
};

type FormSection = {
  logical_name: string;
  display_name: string;
  position: number;
  visible: boolean;
  columns: number;
  fields: FormFieldPlacement[];
  subgrids: FormSubgrid[];
};

type FormTab = {
  logical_name: string;
  display_name: string;
  position: number;
  visible: boolean;
  sections: FormSection[];
};

type SelectionState =
  | { kind: "tab"; tabIndex: number }
  | { kind: "section"; tabIndex: number; sectionIndex: number }
  | {
      kind: "field";
      tabIndex: number;
      sectionIndex: number;
      fieldIndex: number;
    };

function normalizeTabs(input: unknown[] | undefined): FormTab[] {
  if (!Array.isArray(input) || input.length === 0) {
    return [createDefaultTab(0)];
  }

  return input.map((candidate, tabIndex) => {
    const tab = (candidate ?? {}) as Partial<FormTab>;
    const sections = Array.isArray(tab.sections)
      ? tab.sections.map((sectionCandidate, sectionIndex) => {
          const section = (sectionCandidate ?? {}) as Partial<FormSection>;
          const fields = Array.isArray(section.fields)
            ? section.fields.map((fieldCandidate, fieldIndex) => {
                const field = (fieldCandidate ?? {}) as Partial<FormFieldPlacement>;
                return {
                  field_logical_name:
                    typeof field.field_logical_name === "string"
                      ? field.field_logical_name
                      : `field_${tabIndex}_${sectionIndex}_${fieldIndex}`,
                  column:
                    typeof field.column === "number" && field.column >= 0 ? field.column : 0,
                  position: typeof field.position === "number" ? field.position : fieldIndex,
                  visible: field.visible ?? true,
                  read_only: field.read_only ?? false,
                  required_override:
                    typeof field.required_override === "boolean"
                      ? field.required_override
                      : null,
                  label_override:
                    typeof field.label_override === "string" ? field.label_override : null,
                } satisfies FormFieldPlacement;
              })
            : [];

          const subgrids = Array.isArray((section as { subgrids?: unknown[] }).subgrids)
            ? ((section as { subgrids?: unknown[] }).subgrids ?? []).map(
                (subgridCandidate, subgridIndex) => {
                  const subgrid = (subgridCandidate ?? {}) as Partial<FormSubgrid>;
                  return {
                    logical_name:
                      typeof subgrid.logical_name === "string"
                        ? subgrid.logical_name
                        : `subgrid_${tabIndex + 1}_${sectionIndex + 1}_${subgridIndex + 1}`,
                    display_name:
                      typeof subgrid.display_name === "string"
                        ? subgrid.display_name
                        : `Sub-grid ${subgridIndex + 1}`,
                    target_entity_logical_name:
                      typeof subgrid.target_entity_logical_name === "string"
                        ? subgrid.target_entity_logical_name
                        : "",
                    relation_field_logical_name:
                      typeof subgrid.relation_field_logical_name === "string"
                        ? subgrid.relation_field_logical_name
                        : "",
                    position:
                      typeof subgrid.position === "number"
                        ? subgrid.position
                        : subgridIndex,
                    columns: Array.isArray(subgrid.columns)
                      ? subgrid.columns
                          .filter((value): value is string => typeof value === "string")
                          .map((value) => value.trim())
                          .filter((value) => value.length > 0)
                      : [],
                  } satisfies FormSubgrid;
                },
              )
            : [];

          return {
            logical_name:
              typeof section.logical_name === "string"
                ? section.logical_name
                : `section_${tabIndex + 1}_${sectionIndex + 1}`,
            display_name:
              typeof section.display_name === "string"
                ? section.display_name
                : `Section ${sectionIndex + 1}`,
            position: typeof section.position === "number" ? section.position : sectionIndex,
            visible: section.visible ?? true,
            columns:
              typeof section.columns === "number" && [1, 2, 3].includes(section.columns)
                ? section.columns
                : 2,
            fields,
            subgrids,
          } satisfies FormSection;
        })
      : [createDefaultSection(0, 0)];

    return {
      logical_name:
        typeof tab.logical_name === "string" ? tab.logical_name : `tab_${tabIndex + 1}`,
      display_name:
        typeof tab.display_name === "string" ? tab.display_name : `Tab ${tabIndex + 1}`,
      position: typeof tab.position === "number" ? tab.position : tabIndex,
      visible: tab.visible ?? true,
      sections,
    } satisfies FormTab;
  });
}

function createDefaultSection(tabIndex: number, sectionIndex: number): FormSection {
  return {
    logical_name: `section_${tabIndex + 1}_${sectionIndex + 1}`,
    display_name: `Section ${sectionIndex + 1}`,
    position: sectionIndex,
    visible: true,
    columns: 2,
    fields: [],
    subgrids: [],
  };
}

function createDefaultTab(tabIndex: number): FormTab {
  return {
    logical_name: `tab_${tabIndex + 1}`,
    display_name: `Tab ${tabIndex + 1}`,
    position: tabIndex,
    visible: true,
    sections: [createDefaultSection(tabIndex, 0)],
  };
}

function reorderPositions(tabs: FormTab[]): FormTab[] {
  return tabs.map((tab, tabIndex) => ({
    ...tab,
    position: tabIndex,
    sections: tab.sections.map((section, sectionIndex) => ({
      ...section,
      position: sectionIndex,
      fields: section.fields.map((field, fieldIndex) => ({ ...field, position: fieldIndex })),
      subgrids: section.subgrids.map((subgrid, subgridIndex) => ({
        ...subgrid,
        position: subgridIndex,
      })),
    })),
  }));
}

function reorderByIndices<T>(items: T[], sourceIndex: number, targetIndex: number): T[] {
  if (sourceIndex === targetIndex) {
    return items;
  }

  const next = [...items];
  const [entry] = next.splice(sourceIndex, 1);
  next.splice(targetIndex, 0, entry);
  return next;
}

function normalizeHeaderFields(input: string): string[] {
  return input
    .split(",")
    .map((value) => value.trim())
    .filter((value, index, values) => value.length > 0 && values.indexOf(value) === index);
}

export function FormDesignerPanel({
  entityLogicalName,
  initialForm,
  initialForms,
  publishedSchema,
}: FormDesignerPanelProps) {
  const router = useRouter();
  const isEditMode = initialForm !== null;
  const [logicalName, setLogicalName] = useState(initialForm?.logical_name ?? "main_form");
  const [displayName, setDisplayName] = useState(initialForm?.display_name ?? "Main Form");
  const [formType, setFormType] = useState<FormTypeValue>(
    (initialForm?.form_type as FormTypeValue | undefined) ?? "main",
  );
  const [tabs, setTabs] = useState<FormTab[]>(() => normalizeTabs(initialForm?.tabs));
  const [history, setHistory] = useState<FormTab[][]>([]);
  const [future, setFuture] = useState<FormTab[][]>([]);
  const [activeDropLineId, setActiveDropLineId] = useState<string | null>(null);
  const [dragLabel, setDragLabel] = useState<string | null>(null);
  const [isShortcutHelpOpen, setIsShortcutHelpOpen] = useState(false);
  const [activeTabIndex, setActiveTabIndex] = useState(0);
  const [selection, setSelection] = useState<SelectionState>({ kind: "tab", tabIndex: 0 });
  const [headerFieldsText, setHeaderFieldsText] = useState(
    initialForm?.header_fields.join(", ") ?? "",
  );
  const [paletteQuery, setPaletteQuery] = useState("");
  const [isPreviewMode, setIsPreviewMode] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const publishedFields = useMemo(
    () => publishedSchema?.fields ?? [],
    [publishedSchema],
  );
  const hasPublishedSchema = publishedSchema !== null;

  const initialSnapshot = useMemo(
    () =>
      JSON.stringify({
        logical_name: initialForm?.logical_name ?? "main_form",
        display_name: initialForm?.display_name ?? "Main Form",
        form_type: (initialForm?.form_type as FormTypeValue | undefined) ?? "main",
        tabs: normalizeTabs(initialForm?.tabs),
        header_fields: initialForm?.header_fields ?? [],
      }),
    [initialForm],
  );

  const currentSnapshot = useMemo(
    () =>
      JSON.stringify({
        logical_name: logicalName,
        display_name: displayName,
        form_type: formType,
        tabs: reorderPositions(tabs),
        header_fields: normalizeHeaderFields(headerFieldsText),
      }),
    [displayName, formType, headerFieldsText, logicalName, tabs],
  );

  const hasDraftChanges = currentSnapshot !== initialSnapshot;

  useEffect(() => {
    function onKeyDown(event: globalThis.KeyboardEvent) {
      if (!event.metaKey && !event.ctrlKey && !event.altKey && event.key === "?") {
        if (isEditableTarget(event.target)) {
          return;
        }
        event.preventDefault();
        setIsShortcutHelpOpen((current) => !current);
        return;
      }

      if (event.key === "Escape") {
        setIsShortcutHelpOpen(false);
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, []);

  const filteredPaletteFields = useMemo(() => {
    const query = paletteQuery.trim().toLowerCase();
    if (!query) {
      return publishedFields;
    }

    return publishedFields.filter((field) => {
      const haystack = `${field.logical_name} ${field.display_name} ${field.field_type}`.toLowerCase();
      return haystack.includes(query);
    });
  }, [paletteQuery, publishedFields]);

  const placedFieldNames = useMemo(() => {
    const names = new Set<string>();
    for (const tab of tabs) {
      for (const section of tab.sections) {
        for (const field of section.fields) {
          names.add(field.field_logical_name);
        }
      }
    }
    return names;
  }, [tabs]);

  const activeTab = tabs[activeTabIndex] ?? tabs[0] ?? createDefaultTab(0);

  function updateTabs(
    mutator: (current: FormTab[]) => FormTab[],
    options: { trackHistory?: boolean } = {},
  ): void {
    const trackHistory = options.trackHistory ?? true;
    setTabs((current) => {
      const next = reorderPositions(mutator(current));
      if (trackHistory && JSON.stringify(next) !== JSON.stringify(current)) {
        setHistory((previous) => [...previous.slice(-49), current]);
        setFuture([]);
      }
      return next;
    });
  }

  function undo(): void {
    const previous = history.at(-1);
    if (!previous) {
      return;
    }

    setHistory((current) => current.slice(0, -1));
    setFuture((current) => [tabs, ...current].slice(0, 50));
    setTabs(previous);
    setSelection({ kind: "tab", tabIndex: 0 });
    setActiveTabIndex(0);
  }

  function redo(): void {
    const next = future.at(0);
    if (!next) {
      return;
    }

    setFuture((current) => current.slice(1));
    setHistory((current) => [...current, tabs].slice(-50));
    setTabs(next);
    setSelection({ kind: "tab", tabIndex: 0 });
    setActiveTabIndex(0);
  }

  function addTab(): void {
    updateTabs((current) => [...current, createDefaultTab(current.length)]);
    setActiveTabIndex(tabs.length);
    setSelection({ kind: "tab", tabIndex: tabs.length });
  }

  function addSectionToActiveTab(): void {
    updateTabs((current) =>
      current.map((tab, tabIndex) => {
        if (tabIndex !== activeTabIndex) {
          return tab;
        }

        return {
          ...tab,
          sections: [...tab.sections, createDefaultSection(tabIndex, tab.sections.length)],
        };
      }),
    );
  }

  function addFieldToSection(
    fieldLogicalName: string,
    tabIndex: number,
    sectionIndex: number,
    column: number,
  ): void {
    placeFieldInSection(fieldLogicalName, tabIndex, sectionIndex, column, null, "palette");
  }

  function moveSelectionByOffset(offset: number): void {
    if (selection.kind === "tab") {
      const targetIndex = selection.tabIndex + offset;
      if (targetIndex < 0 || targetIndex >= tabs.length) {
        return;
      }
      updateTabs((current) => reorderByIndices(current, selection.tabIndex, targetIndex));
      setActiveTabIndex(targetIndex);
      setSelection({ kind: "tab", tabIndex: targetIndex });
      return;
    }

    if (selection.kind === "section") {
      const tab = tabs[selection.tabIndex];
      if (!tab) {
        return;
      }

      const targetIndex = selection.sectionIndex + offset;
      if (targetIndex < 0 || targetIndex >= tab.sections.length) {
        return;
      }

      updateTabs((current) =>
        current.map((currentTab, tabIndex) => {
          if (tabIndex !== selection.tabIndex) {
            return currentTab;
          }

          return {
            ...currentTab,
            sections: reorderByIndices(
              currentTab.sections,
              selection.sectionIndex,
              targetIndex,
            ),
          };
        }),
      );
      setSelection({
        kind: "section",
        tabIndex: selection.tabIndex,
        sectionIndex: targetIndex,
      });
      return;
    }

    if (selection.kind !== "field" || !selectedField || !selectedSection) {
      return;
    }

    const fieldsInColumn = selectedSection.fields
      .filter((field) => field.column === selectedField.column)
      .sort((left, right) => left.position - right.position);
    const currentColumnIndex = fieldsInColumn.findIndex(
      (field) =>
        field.field_logical_name === selectedField.field_logical_name &&
        field.position === selectedField.position,
    );
    if (currentColumnIndex < 0) {
      return;
    }

    const targetColumnIndex = currentColumnIndex + offset;
    if (targetColumnIndex < 0 || targetColumnIndex >= fieldsInColumn.length) {
      return;
    }

    placeFieldInSection(
      selectedField.field_logical_name,
      selection.tabIndex,
      selection.sectionIndex,
      selectedField.column,
      targetColumnIndex,
      "canvas",
    );
    setSelection({
      kind: "section",
      tabIndex: selection.tabIndex,
      sectionIndex: selection.sectionIndex,
    });
  }

  function handleCanvasKeyDown(event: KeyboardEvent<HTMLDivElement>): void {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "z") {
      event.preventDefault();
      if (event.shiftKey) {
        redo();
      } else {
        undo();
      }
      return;
    }

    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "y") {
      event.preventDefault();
      redo();
      return;
    }

    if (!event.altKey) {
      return;
    }

    if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
      event.preventDefault();
      moveSelectionByOffset(-1);
      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowRight") {
      event.preventDefault();
      moveSelectionByOffset(1);
    }
  }

  function placeFieldInSection(
    fieldLogicalName: string,
    tabIndex: number,
    sectionIndex: number,
    column: number,
    insertAt: number | null,
    source: "palette" | "canvas",
  ): void {
    if (source === "palette" && placedFieldNames.has(fieldLogicalName)) {
      setErrorMessage(`Field '${fieldLogicalName}' is already placed in this form.`);
      return;
    }

    setErrorMessage(null);
    updateTabs((current) => {
      let movingField: FormFieldPlacement | null = null;
      const withoutMoving = current.map((tab) => ({
        ...tab,
        sections: tab.sections.map((section) => {
          const nextFields = section.fields.filter((field) => {
            if (field.field_logical_name !== fieldLogicalName) {
              return true;
            }
            movingField = field;
            return false;
          });
          return {
            ...section,
            fields: nextFields,
          };
        }),
      }));

      return withoutMoving.map((tab, currentTabIndex) => {
        if (currentTabIndex !== tabIndex) {
          return tab;
        }

        return {
          ...tab,
          sections: tab.sections.map((section, currentSectionIndex) => {
            if (currentSectionIndex !== sectionIndex) {
              return section;
            }

            const nextField: FormFieldPlacement = movingField
              ? {
                  ...movingField,
                  column,
                }
              : {
                  field_logical_name: fieldLogicalName,
                  column,
                  position: section.fields.length,
                  visible: true,
                  read_only: false,
                  required_override: null,
                  label_override: null,
                };

            const targetColumnFields = section.fields
              .filter((field) => field.column === column)
              .sort((left, right) => left.position - right.position);
            const otherFields = section.fields.filter((field) => field.column !== column);
            const targetIndex =
              insertAt === null
                ? targetColumnFields.length
                : Math.max(0, Math.min(insertAt, targetColumnFields.length));
            const nextTargetColumnFields = [...targetColumnFields];
            nextTargetColumnFields.splice(targetIndex, 0, nextField);

            return {
              ...section,
              fields: [...otherFields, ...nextTargetColumnFields],
            };
          }),
        };
      });
    });

    setSelection({ kind: "section", tabIndex, sectionIndex });
  }

  function deleteField(tabIndex: number, sectionIndex: number, fieldIndex: number): void {
    updateTabs((current) =>
      current.map((tab, currentTabIndex) => {
        if (currentTabIndex !== tabIndex) {
          return tab;
        }

        return {
          ...tab,
          sections: tab.sections.map((section, currentSectionIndex) => {
            if (currentSectionIndex !== sectionIndex) {
              return section;
            }

            return {
              ...section,
              fields: section.fields.filter((_, currentFieldIndex) => currentFieldIndex !== fieldIndex),
            };
          }),
        };
      }),
    );
    setSelection({ kind: "section", tabIndex, sectionIndex });
  }

  function validateQuickCreateShape(nextTabs: FormTab[], nextFormType: FormTypeValue): string | null {
    if (nextFormType !== "quick_create") {
      return null;
    }

    if (nextTabs.length !== 1 || (nextTabs[0]?.sections.length ?? 0) !== 1) {
      return "Quick Create forms require exactly one tab and one section.";
    }

    return null;
  }

  async function handleSave(): Promise<void> {
    setStatusMessage(null);
    setErrorMessage(null);

    if (!hasPublishedSchema) {
      setErrorMessage("Publish the entity schema before saving forms.");
      return;
    }

    const quickCreateError = validateQuickCreateShape(tabs, formType);
    if (quickCreateError) {
      setErrorMessage(quickCreateError);
      return;
    }

    setIsSaving(true);
    try {
      const payload: CreateFormRequest = {
        logical_name: logicalName,
        display_name: displayName,
        form_type: formType,
        tabs: tabs as unknown[],
        header_fields: normalizeHeaderFields(headerFieldsText),
      };

      const path = isEditMode
        ? `/api/entities/${entityLogicalName}/forms/${initialForm.logical_name}`
        : `/api/entities/${entityLogicalName}/forms`;
      const response = await apiFetch(path, {
        method: isEditMode ? "PUT" : "POST",
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const payload = (await response.json()) as { message?: string };
        setErrorMessage(payload.message ?? "Unable to save form.");
        return;
      }

      setStatusMessage("Form saved.");
      if (!isEditMode) {
        router.replace(`/maker/entities/${encodeURIComponent(entityLogicalName)}/forms/${encodeURIComponent(logicalName)}`);
      } else {
        router.refresh();
      }
    } catch {
      setErrorMessage("Unable to save form.");
    } finally {
      setIsSaving(false);
    }
  }

  const selectedField =
    selection.kind === "field"
      ? tabs[selection.tabIndex]?.sections[selection.sectionIndex]?.fields[selection.fieldIndex] ?? null
      : null;
  const selectedSection =
    selection.kind === "section" || selection.kind === "field"
      ? tabs[selection.tabIndex]?.sections[selection.sectionIndex] ?? null
      : null;
  const selectedTab = selection.kind === "tab" ? tabs[selection.tabIndex] ?? null : null;

  function updateSelectedTab(patch: Partial<FormTab>): void {
    if (!selectedTab || selection.kind !== "tab") {
      return;
    }
    const tabIndex = selection.tabIndex;
    updateTabs((current) =>
      current.map((tab, currentTabIndex) => (currentTabIndex === tabIndex ? { ...tab, ...patch } : tab)),
    );
  }

  function updateSelectedSection(patch: Partial<FormSection>): void {
    if (!selectedSection || (selection.kind !== "section" && selection.kind !== "field")) {
      return;
    }
    const tabIndex = selection.tabIndex;
    const sectionIndex = selection.sectionIndex;
    updateTabs((current) =>
      current.map((tab, currentTabIndex) => {
        if (currentTabIndex !== tabIndex) {
          return tab;
        }
        return {
          ...tab,
          sections: tab.sections.map((section, currentSectionIndex) =>
            currentSectionIndex === sectionIndex ? { ...section, ...patch } : section,
          ),
        };
      }),
    );
  }

  function updateSelectedField(patch: Partial<FormFieldPlacement>): void {
    if (!selectedField || selection.kind !== "field") {
      return;
    }
    const { tabIndex, sectionIndex, fieldIndex } = selection;
    updateTabs((current) =>
      current.map((tab, currentTabIndex) => {
        if (currentTabIndex !== tabIndex) {
          return tab;
        }
        return {
          ...tab,
          sections: tab.sections.map((section, currentSectionIndex) => {
            if (currentSectionIndex !== sectionIndex) {
              return section;
            }
            return {
              ...section,
              fields: section.fields.map((field, currentFieldIndex) =>
                currentFieldIndex === fieldIndex ? { ...field, ...patch } : field,
              ),
            };
          }),
        };
      }),
    );
  }

  function addSubgridToSelectedSection(): void {
    if (!selectedSection || (selection.kind !== "section" && selection.kind !== "field")) {
      return;
    }

    const tabIndex = selection.tabIndex;
    const sectionIndex = selection.sectionIndex;
    const nextIndex = selectedSection.subgrids.length;

    updateTabs((current) =>
      current.map((tab, currentTabIndex) => {
        if (currentTabIndex !== tabIndex) {
          return tab;
        }

        return {
          ...tab,
          sections: tab.sections.map((section, currentSectionIndex) => {
            if (currentSectionIndex !== sectionIndex) {
              return section;
            }

            return {
              ...section,
              subgrids: [
                ...section.subgrids,
                {
                  logical_name: `subgrid_${tabIndex + 1}_${sectionIndex + 1}_${nextIndex + 1}`,
                  display_name: `Sub-grid ${nextIndex + 1}`,
                  target_entity_logical_name: "",
                  relation_field_logical_name: "",
                  position: nextIndex,
                  columns: [],
                },
              ],
            };
          }),
        };
      }),
    );
  }

  function updateSubgridInSelectedSection(
    subgridIndex: number,
    patch: Partial<FormSubgrid>,
  ): void {
    if (!selectedSection || (selection.kind !== "section" && selection.kind !== "field")) {
      return;
    }

    const tabIndex = selection.tabIndex;
    const sectionIndex = selection.sectionIndex;

    updateTabs((current) =>
      current.map((tab, currentTabIndex) => {
        if (currentTabIndex !== tabIndex) {
          return tab;
        }

        return {
          ...tab,
          sections: tab.sections.map((section, currentSectionIndex) => {
            if (currentSectionIndex !== sectionIndex) {
              return section;
            }

            return {
              ...section,
              subgrids: section.subgrids.map((subgrid, currentSubgridIndex) =>
                currentSubgridIndex === subgridIndex ? { ...subgrid, ...patch } : subgrid,
              ),
            };
          }),
        };
      }),
    );
  }

  function removeSubgridFromSelectedSection(subgridIndex: number): void {
    if (!selectedSection || (selection.kind !== "section" && selection.kind !== "field")) {
      return;
    }

    const tabIndex = selection.tabIndex;
    const sectionIndex = selection.sectionIndex;

    updateTabs((current) =>
      current.map((tab, currentTabIndex) => {
        if (currentTabIndex !== tabIndex) {
          return tab;
        }

        return {
          ...tab,
          sections: tab.sections.map((section, currentSectionIndex) => {
            if (currentSectionIndex !== sectionIndex) {
              return section;
            }

            return {
              ...section,
              subgrids: section.subgrids.filter((_, currentSubgridIndex) => currentSubgridIndex !== subgridIndex),
            };
          }),
        };
      }),
    );
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
          <div className="space-y-2">
            <CardTitle>{isEditMode ? "Form Designer" : "New Form"}</CardTitle>
            <CardDescription>
              Design tab/section/column layout, configure field behavior, and preview worker form rendering.
            </CardDescription>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <StatusBadge tone="neutral">Forms {initialForms.length}</StatusBadge>
            <StatusBadge tone={hasPublishedSchema ? "success" : "warning"}>
              {hasPublishedSchema ? "Published schema ready" : "Publish required"}
            </StatusBadge>
            <StatusBadge tone={hasDraftChanges ? "warning" : "neutral"}>
              {hasDraftChanges ? "Draft changes" : "Draft saved"}
            </StatusBadge>
            <Button type="button" variant={isPreviewMode ? "default" : "outline"} onClick={() => setIsPreviewMode((current) => !current)}>
              {isPreviewMode ? "Exit Preview" : "Preview Mode"}
            </Button>
            <Button type="button" variant="outline" onClick={undo} disabled={history.length === 0}>
              Undo
            </Button>
            <Button type="button" variant="outline" onClick={redo} disabled={future.length === 0}>
              Redo
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={() => setIsShortcutHelpOpen((current) => !current)}
              title="Toggle shortcuts (?)"
            >
              Shortcuts
            </Button>
            <Button type="button" disabled={isSaving} onClick={handleSave}>
              {isSaving ? "Saving..." : "Save Form"}
            </Button>
          </div>
        </CardHeader>
        <CardContent className="grid gap-3 md:grid-cols-4">
          <div className="space-y-2 md:col-span-1">
            <Label htmlFor="form_logical_name">Logical Name</Label>
            <Input
              id="form_logical_name"
              value={logicalName}
              onChange={(event) => setLogicalName(event.target.value)}
              disabled={isEditMode}
            />
          </div>
          <div className="space-y-2 md:col-span-1">
            <Label htmlFor="form_display_name">Display Name</Label>
            <Input
              id="form_display_name"
              value={displayName}
              onChange={(event) => setDisplayName(event.target.value)}
            />
          </div>
          <div className="space-y-2 md:col-span-1">
            <Label htmlFor="form_type">Form Type</Label>
            <Select
              id="form_type"
              value={formType}
              onChange={(event) => setFormType(event.target.value as FormTypeValue)}
            >
              <option value="main">Main</option>
              <option value="quick_create">Quick Create</option>
              <option value="quick_view">Quick View</option>
            </Select>
          </div>
          <div className="space-y-2 md:col-span-1">
            <Label htmlFor="header_fields">Header Fields (comma-separated)</Label>
            <Input
              id="header_fields"
              value={headerFieldsText}
              onChange={(event) => setHeaderFieldsText(event.target.value)}
              placeholder="name, status, owner"
            />
          </div>
        </CardContent>
      </Card>

      <div className="grid gap-4 xl:grid-cols-[260px_1fr_320px]">
        <Card className="h-fit">
          <CardHeader>
            <CardTitle className="text-base">Field Palette</CardTitle>
            <CardDescription>Drag fields into section columns, or use quick add in drop zones.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <Input
              value={paletteQuery}
              onChange={(event) => setPaletteQuery(event.target.value)}
              placeholder="Search field palette"
            />
            <div className="max-h-[500px] space-y-2 overflow-y-auto">
              {filteredPaletteFields.map((field) => (
                <button
                  key={field.logical_name}
                  type="button"
                  draggable
                  onDragStart={(event) => {
                    event.dataTransfer.setData("text/plain", field.logical_name);
                    event.dataTransfer.setData("text/form-field-source", "palette");
                    setDragLabel(field.display_name || field.logical_name);
                  }}
                  onDragEnd={() => setDragLabel(null)}
                  className="w-full rounded-md border border-zinc-200 bg-white px-3 py-2 text-left text-sm hover:border-emerald-300"
                >
                  <p className="font-medium text-zinc-900">{field.display_name}</p>
                  <p className="font-mono text-xs text-zinc-500">
                    {field.logical_name} · {field.field_type}
                  </p>
                  {placedFieldNames.has(field.logical_name) ? (
                    <p className="mt-1 text-xs text-emerald-700">Placed</p>
                  ) : null}
                </button>
              ))}
              {filteredPaletteFields.length === 0 ? (
                <p className="text-xs text-zinc-500">No fields match this filter.</p>
              ) : null}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
            <div>
              <CardTitle className="text-base">Canvas</CardTitle>
              <CardDescription>
                Tabs, sections, and columns are rendered as worker-surface layout blocks.
              </CardDescription>
            </div>
            <div className="flex gap-2">
              <Button type="button" variant="outline" onClick={addTab}>
                Add Tab
              </Button>
              <Button type="button" variant="outline" onClick={addSectionToActiveTab}>
                Add Section
              </Button>
            </div>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex flex-wrap gap-2">
              {tabs.map((tab, index) => (
                <Button
                  key={`${tab.logical_name}-${index}`}
                  type="button"
                  variant={index === activeTabIndex ? "default" : "outline"}
                  onClick={() => {
                    setActiveTabIndex(index);
                    setSelection({ kind: "tab", tabIndex: index });
                  }}
                  draggable
                  onDragStart={(event) => {
                    event.dataTransfer.setData("text/tab-index", String(index));
                  }}
                  onDragOver={(event) => event.preventDefault()}
                  onDrop={(event) => {
                    const sourceIndex = Number.parseInt(
                      event.dataTransfer.getData("text/tab-index"),
                      10,
                    );
                    if (Number.isNaN(sourceIndex) || sourceIndex === index) {
                      return;
                    }
                    updateTabs((current) => {
                      const next = [...current];
                      const [moved] = next.splice(sourceIndex, 1);
                      next.splice(index, 0, moved);
                      return next;
                    });
                    setActiveTabIndex(index);
                    setSelection({ kind: "tab", tabIndex: index });
                  }}
                >
                  {tab.display_name}
                </Button>
              ))}
            </div>

            <div className="space-y-3" tabIndex={0} onKeyDown={handleCanvasKeyDown}>
              {activeTab.sections.map((section, sectionIndex) => (
                <div
                  key={`${section.logical_name}-${sectionIndex}`}
                  className="rounded-md border border-zinc-200 bg-zinc-50 p-3"
                  onClick={() =>
                    setSelection({
                      kind: "section",
                      tabIndex: activeTabIndex,
                      sectionIndex,
                    })
                  }
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
                            placeFieldInSection(
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
                                    onSetActiveLine={setActiveDropLineId}
                                    onDrop={(event) => {
                                      const fieldLogicalName = event.dataTransfer.getData("text/plain");
                                      const source =
                                        event.dataTransfer.getData("text/form-field-source") === "canvas"
                                          ? "canvas"
                                          : "palette";
                                      if (!fieldLogicalName) {
                                        return;
                                      }
                                      placeFieldInSection(
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
                                      setDragLabel(metadata?.display_name || field.field_logical_name);
                                    }}
                                    onDragEnd={() => setDragLabel(null)}
                                    className="w-full rounded-md border border-zinc-200 bg-zinc-50 px-2 py-2 text-left"
                                    onClick={(event) => {
                                      event.stopPropagation();
                                      setSelection({
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
                                          deleteField(activeTabIndex, sectionIndex, fieldIndex);
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
                              onSetActiveLine={setActiveDropLineId}
                              onDrop={(event) => {
                                const fieldLogicalName = event.dataTransfer.getData("text/plain");
                                const source =
                                  event.dataTransfer.getData("text/form-field-source") === "canvas"
                                    ? "canvas"
                                    : "palette";
                                if (!fieldLogicalName) {
                                  return;
                                }
                                placeFieldInSection(
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
                                  addFieldToSection(
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
                    onChange={(event) => updateSelectedTab({ display_name: event.target.value })}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="selected_tab_logical_name">Logical Name</Label>
                  <Input
                    id="selected_tab_logical_name"
                    value={selectedTab.logical_name}
                    onChange={(event) => updateSelectedTab({ logical_name: event.target.value })}
                  />
                </div>
                <div className="flex items-center gap-2">
                  <Checkbox
                    id="selected_tab_visible"
                    checked={selectedTab.visible}
                    onChange={(event) => updateSelectedTab({ visible: event.target.checked })}
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
                      updateSelectedSection({ display_name: event.target.value })
                    }
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="selected_section_columns">Columns</Label>
                  <Select
                    id="selected_section_columns"
                    value={String(selectedSection.columns)}
                    onChange={(event) =>
                      updateSelectedSection({
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
                    onChange={(event) => updateSelectedSection({ visible: event.target.checked })}
                  />
                  <Label htmlFor="selected_section_visible">Visible</Label>
                </div>

                <div className="space-y-2 border-t border-zinc-200 pt-3">
                  <div className="flex items-center justify-between">
                    <p className="text-xs font-semibold uppercase tracking-[0.18em] text-zinc-500">
                      Sub-grids
                    </p>
                    <Button type="button" size="sm" variant="outline" onClick={addSubgridToSelectedSection}>
                      Add Sub-grid
                    </Button>
                  </div>

                  {selectedSection.subgrids.length > 0 ? (
                    selectedSection.subgrids
                      .slice()
                      .sort((left, right) => left.position - right.position)
                      .map((subgrid, subgridIndex) => (
                        <div key={`${subgrid.logical_name}-${subgridIndex}`} className="space-y-2 rounded-md border border-zinc-200 p-2">
                          <Input
                            value={subgrid.display_name}
                            onChange={(event) =>
                              updateSubgridInSelectedSection(subgridIndex, {
                                display_name: event.target.value,
                              })
                            }
                            placeholder="Display name"
                          />
                          <Input
                            value={subgrid.logical_name}
                            onChange={(event) =>
                              updateSubgridInSelectedSection(subgridIndex, {
                                logical_name: event.target.value,
                              })
                            }
                            placeholder="Logical name"
                          />
                          <Input
                            value={subgrid.target_entity_logical_name}
                            onChange={(event) =>
                              updateSubgridInSelectedSection(subgridIndex, {
                                target_entity_logical_name: event.target.value,
                              })
                            }
                            placeholder="Target entity logical name"
                          />
                          <Input
                            value={subgrid.relation_field_logical_name}
                            onChange={(event) =>
                              updateSubgridInSelectedSection(subgridIndex, {
                                relation_field_logical_name: event.target.value,
                              })
                            }
                            placeholder="Target relation field logical name"
                          />
                          <Input
                            value={subgrid.columns.join(", ")}
                            onChange={(event) =>
                              updateSubgridInSelectedSection(subgridIndex, {
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
                            onClick={() => removeSubgridFromSelectedSection(subgridIndex)}
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
                      updateSelectedField({
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
                      updateSelectedField({
                        required_override:
                          value === "inherit" ? null : value === "required",
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
                      onChange={(event) =>
                        updateSelectedField({ visible: event.target.checked })
                      }
                    />
                    <Label htmlFor="selected_field_visible">Visible</Label>
                  </div>
                  <div className="flex items-center gap-2">
                    <Checkbox
                      id="selected_field_read_only"
                      checked={selectedField.read_only}
                      onChange={(event) =>
                        updateSelectedField({ read_only: event.target.checked })
                      }
                    />
                    <Label htmlFor="selected_field_read_only">Read Only</Label>
                  </div>
                </div>
              </div>
            ) : null}
          </CardContent>
        </Card>
      </div>

      {!hasPublishedSchema ? (
        <Notice tone="warning">
          This entity does not have a published schema yet. Publish the entity before saving form definitions.
        </Notice>
      ) : null}
      {isShortcutHelpOpen ? (
        <Notice tone="neutral">
          <p className="font-semibold">Form Designer Shortcuts</p>
          <ul className="mt-1 list-disc pl-5 text-sm">
            <li>`?` toggle this help</li>
            <li>`Ctrl/Cmd + Z` undo</li>
            <li>`Ctrl/Cmd + Y` redo</li>
            <li>`Ctrl/Cmd + Shift + Z` redo</li>
            <li>`Alt + Arrow` reorder selected tab/section/field</li>
            <li>`Escape` close this help</li>
          </ul>
        </Notice>
      ) : null}
      {dragLabel ? (
        <Notice tone="neutral">Dragging `{dragLabel}` - drop on highlighted insertion line.</Notice>
      ) : null}
      {errorMessage ? <Notice tone="error">{errorMessage}</Notice> : null}
      {statusMessage ? <Notice tone="success">{statusMessage}</Notice> : null}
    </div>
  );
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  const tagName = target.tagName;
  return tagName === "INPUT" || tagName === "TEXTAREA" || tagName === "SELECT" || target.isContentEditable;
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
