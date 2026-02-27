"use client";

import { type KeyboardEvent, useCallback, useMemo, useState } from "react";

import type {
  FormFieldPlacement,
  FormSection,
  FormSelectionState,
  FormSubgrid,
  FormTab,
  FormTypeValue,
} from "@/components/studio/types";

// ---------------------------------------------------------------------------
// Pure helpers (extracted from form-designer-panel.tsx)
// ---------------------------------------------------------------------------

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
  if (sourceIndex === targetIndex) return items;
  const next = [...items];
  const [entry] = next.splice(sourceIndex, 1);
  next.splice(targetIndex, 0, entry);
  return next;
}

export function normalizeTabs(input: unknown[] | undefined): FormTab[] {
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
                    typeof field.required_override === "boolean" ? field.required_override : null,
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
                      typeof subgrid.position === "number" ? subgrid.position : subgridIndex,
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
      : [createDefaultSection(tabIndex, 0)];

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

export function normalizeHeaderFields(input: string): string[] {
  return input
    .split(",")
    .map((value) => value.trim())
    .filter((value, index, values) => value.length > 0 && values.indexOf(value) === index);
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

export type FormEditorState = {
  tabs: FormTab[];
  activeTabIndex: number;
  activeTab: FormTab;
  selection: FormSelectionState;
  activeDropLineId: string | null;
  dragLabel: string | null;
  placedFieldNames: Set<string>;
  canUndo: boolean;
  canRedo: boolean;

  setActiveTabIndex: (index: number) => void;
  setSelection: (selection: FormSelectionState) => void;
  setActiveDropLineId: (lineId: string | null) => void;
  setDragLabel: (label: string | null) => void;

  updateTabs: (
    mutator: (current: FormTab[]) => FormTab[],
    options?: { trackHistory?: boolean },
  ) => void;
  undo: () => void;
  redo: () => void;
  addTab: () => void;
  addSectionToActiveTab: () => void;
  addFieldToSection: (
    fieldLogicalName: string,
    tabIndex: number,
    sectionIndex: number,
    column: number,
  ) => void;
  placeFieldInSection: (
    fieldLogicalName: string,
    tabIndex: number,
    sectionIndex: number,
    column: number,
    insertAt: number | null,
    source: "palette" | "canvas",
  ) => void;
  deleteField: (tabIndex: number, sectionIndex: number, fieldIndex: number) => void;
  handleCanvasKeyDown: (event: KeyboardEvent<HTMLDivElement>) => void;
  resetFromTabs: (nextTabs: unknown[] | undefined) => void;

  // Property helpers
  selectedTab: FormTab | null;
  selectedSection: FormSection | null;
  selectedField: FormFieldPlacement | null;
  updateSelectedTab: (patch: Partial<FormTab>) => void;
  updateSelectedSection: (patch: Partial<FormSection>) => void;
  updateSelectedField: (patch: Partial<FormFieldPlacement>) => void;
  addSubgridToSelectedSection: () => void;
  updateSubgridInSelectedSection: (subgridIndex: number, patch: Partial<FormSubgrid>) => void;
  removeSubgridFromSelectedSection: (subgridIndex: number) => void;
};

type UseFormEditorStateInput = {
  initialTabs: unknown[] | undefined;
  onError?: (message: string) => void;
};

export function useFormEditorState({
  initialTabs,
  onError,
}: UseFormEditorStateInput): FormEditorState {
  const [tabs, setTabs] = useState<FormTab[]>(() => normalizeTabs(initialTabs));
  const [history, setHistory] = useState<FormTab[][]>([]);
  const [future, setFuture] = useState<FormTab[][]>([]);
  const [activeDropLineId, setActiveDropLineId] = useState<string | null>(null);
  const [dragLabel, setDragLabel] = useState<string | null>(null);
  const [activeTabIndex, setActiveTabIndex] = useState(0);
  const [selection, setSelection] = useState<FormSelectionState>({ kind: "tab", tabIndex: 0 });

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

  const resetFromTabs = useCallback((nextTabsInput: unknown[] | undefined): void => {
    const nextTabs = normalizeTabs(nextTabsInput);
    setTabs(nextTabs);
    setHistory([]);
    setFuture([]);
    setActiveDropLineId(null);
    setDragLabel(null);
    setActiveTabIndex(0);
    setSelection({ kind: "tab", tabIndex: 0 });
  }, []);

  // ---- Tab mutations with undo/redo ----

  const updateTabs = useCallback(
    (
      mutator: (current: FormTab[]) => FormTab[],
      options: { trackHistory?: boolean } = {},
    ): void => {
      const trackHistory = options.trackHistory ?? true;
      setTabs((current) => {
        const next = reorderPositions(mutator(current));
        if (trackHistory && JSON.stringify(next) !== JSON.stringify(current)) {
          setHistory((previous) => [...previous.slice(-49), current]);
          setFuture([]);
        }
        return next;
      });
    },
    [],
  );

  function undo(): void {
    const previous = history.at(-1);
    if (!previous) return;
    setHistory((current) => current.slice(0, -1));
    setFuture((current) => [tabs, ...current].slice(0, 50));
    setTabs(previous);
    setSelection({ kind: "tab", tabIndex: 0 });
    setActiveTabIndex(0);
  }

  function redo(): void {
    const next = future.at(0);
    if (!next) return;
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
        if (tabIndex !== activeTabIndex) return tab;
        return {
          ...tab,
          sections: [...tab.sections, createDefaultSection(tabIndex, tab.sections.length)],
        };
      }),
    );
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
      onError?.(`Field '${fieldLogicalName}' is already placed in this form.`);
      return;
    }

    updateTabs((current) => {
      let movingField: FormFieldPlacement | null = null;
      const withoutMoving = current.map((tab) => ({
        ...tab,
        sections: tab.sections.map((section) => {
          const nextFields = section.fields.filter((field) => {
            if (field.field_logical_name !== fieldLogicalName) return true;
            movingField = field;
            return false;
          });
          return { ...section, fields: nextFields };
        }),
      }));

      return withoutMoving.map((tab, currentTabIndex) => {
        if (currentTabIndex !== tabIndex) return tab;
        return {
          ...tab,
          sections: tab.sections.map((section, currentSectionIndex) => {
            if (currentSectionIndex !== sectionIndex) return section;

            const nextField: FormFieldPlacement = movingField
              ? { ...movingField, column }
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

            return { ...section, fields: [...otherFields, ...nextTargetColumnFields] };
          }),
        };
      });
    });
    setSelection({ kind: "section", tabIndex, sectionIndex });
  }

  function addFieldToSection(
    fieldLogicalName: string,
    tabIndex: number,
    sectionIndex: number,
    column: number,
  ): void {
    placeFieldInSection(fieldLogicalName, tabIndex, sectionIndex, column, null, "palette");
  }

  function deleteField(tabIndex: number, sectionIndex: number, fieldIndex: number): void {
    updateTabs((current) =>
      current.map((tab, currentTabIndex) => {
        if (currentTabIndex !== tabIndex) return tab;
        return {
          ...tab,
          sections: tab.sections.map((section, currentSectionIndex) => {
            if (currentSectionIndex !== sectionIndex) return section;
            return {
              ...section,
              fields: section.fields.filter((_, i) => i !== fieldIndex),
            };
          }),
        };
      }),
    );
    setSelection({ kind: "section", tabIndex, sectionIndex });
  }

  // ---- Keyboard-driven reorder ----

  const selectedField =
    selection.kind === "field"
      ? tabs[selection.tabIndex]?.sections[selection.sectionIndex]?.fields[selection.fieldIndex] ??
        null
      : null;
  const selectedSection =
    selection.kind === "section" || selection.kind === "field"
      ? tabs[selection.tabIndex]?.sections[selection.sectionIndex] ?? null
      : null;
  const selectedTab = selection.kind === "tab" ? tabs[selection.tabIndex] ?? null : null;

  function moveSelectionByOffset(offset: number): void {
    if (selection.kind === "tab") {
      const targetIndex = selection.tabIndex + offset;
      if (targetIndex < 0 || targetIndex >= tabs.length) return;
      updateTabs((current) => reorderByIndices(current, selection.tabIndex, targetIndex));
      setActiveTabIndex(targetIndex);
      setSelection({ kind: "tab", tabIndex: targetIndex });
      return;
    }

    if (selection.kind === "section") {
      const tab = tabs[selection.tabIndex];
      if (!tab) return;
      const targetIndex = selection.sectionIndex + offset;
      if (targetIndex < 0 || targetIndex >= tab.sections.length) return;
      updateTabs((current) =>
        current.map((currentTab, tabIndex) => {
          if (tabIndex !== selection.tabIndex) return currentTab;
          return {
            ...currentTab,
            sections: reorderByIndices(currentTab.sections, selection.sectionIndex, targetIndex),
          };
        }),
      );
      setSelection({ kind: "section", tabIndex: selection.tabIndex, sectionIndex: targetIndex });
      return;
    }

    if (selection.kind !== "field" || !selectedField || !selectedSection) return;

    const fieldsInColumn = selectedSection.fields
      .filter((f) => f.column === selectedField.column)
      .sort((a, b) => a.position - b.position);
    const currentColumnIndex = fieldsInColumn.findIndex(
      (f) =>
        f.field_logical_name === selectedField.field_logical_name &&
        f.position === selectedField.position,
    );
    if (currentColumnIndex < 0) return;

    const targetColumnIndex = currentColumnIndex + offset;
    if (targetColumnIndex < 0 || targetColumnIndex >= fieldsInColumn.length) return;

    placeFieldInSection(
      selectedField.field_logical_name,
      selection.tabIndex,
      selection.sectionIndex,
      selectedField.column,
      targetColumnIndex,
      "canvas",
    );
    setSelection({ kind: "section", tabIndex: selection.tabIndex, sectionIndex: selection.sectionIndex });
  }

  function handleCanvasKeyDown(event: KeyboardEvent<HTMLDivElement>): void {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "z") {
      event.preventDefault();
      if (event.shiftKey) redo();
      else undo();
      return;
    }
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "y") {
      event.preventDefault();
      redo();
      return;
    }
    if (!event.altKey) return;
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

  // ---- Property update helpers (from use-form-designer-properties) ----

  const updateSelectedTab = useCallback(
    (patch: Partial<FormTab>): void => {
      if (!selectedTab || selection.kind !== "tab") return;
      const tabIndex = selection.tabIndex;
      updateTabs((current) =>
        current.map((tab, i) => (i === tabIndex ? { ...tab, ...patch } : tab)),
      );
    },
    [selectedTab, selection, updateTabs],
  );

  const updateSelectedSection = useCallback(
    (patch: Partial<FormSection>): void => {
      if (!selectedSection || (selection.kind !== "section" && selection.kind !== "field")) return;
      const tabIndex = selection.tabIndex;
      const sectionIndex = selection.sectionIndex;
      updateTabs((current) =>
        current.map((tab, ti) => {
          if (ti !== tabIndex) return tab;
          return {
            ...tab,
            sections: tab.sections.map((section, si) =>
              si === sectionIndex ? { ...section, ...patch } : section,
            ),
          };
        }),
      );
    },
    [selectedSection, selection, updateTabs],
  );

  const updateSelectedField = useCallback(
    (patch: Partial<FormFieldPlacement>): void => {
      if (!selectedField || selection.kind !== "field") return;
      const { tabIndex, sectionIndex, fieldIndex } = selection;
      updateTabs((current) =>
        current.map((tab, ti) => {
          if (ti !== tabIndex) return tab;
          return {
            ...tab,
            sections: tab.sections.map((section, si) => {
              if (si !== sectionIndex) return section;
              return {
                ...section,
                fields: section.fields.map((field, fi) =>
                  fi === fieldIndex ? { ...field, ...patch } : field,
                ),
              };
            }),
          };
        }),
      );
    },
    [selectedField, selection, updateTabs],
  );

  const addSubgridToSelectedSection = useCallback((): void => {
    if (!selectedSection || (selection.kind !== "section" && selection.kind !== "field")) return;
    const tabIndex = selection.tabIndex;
    const sectionIndex = selection.sectionIndex;
    const nextIndex = selectedSection.subgrids.length;
    updateTabs((current) =>
      current.map((tab, ti) => {
        if (ti !== tabIndex) return tab;
        return {
          ...tab,
          sections: tab.sections.map((section, si) => {
            if (si !== sectionIndex) return section;
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
  }, [selectedSection, selection, updateTabs]);

  const updateSubgridInSelectedSection = useCallback(
    (subgridIndex: number, patch: Partial<FormSubgrid>): void => {
      if (!selectedSection || (selection.kind !== "section" && selection.kind !== "field")) return;
      const tabIndex = selection.tabIndex;
      const sectionIndex = selection.sectionIndex;
      updateTabs((current) =>
        current.map((tab, ti) => {
          if (ti !== tabIndex) return tab;
          return {
            ...tab,
            sections: tab.sections.map((section, si) => {
              if (si !== sectionIndex) return section;
              return {
                ...section,
                subgrids: section.subgrids.map((subgrid, i) =>
                  i === subgridIndex ? { ...subgrid, ...patch } : subgrid,
                ),
              };
            }),
          };
        }),
      );
    },
    [selectedSection, selection, updateTabs],
  );

  const removeSubgridFromSelectedSection = useCallback(
    (subgridIndex: number): void => {
      if (!selectedSection || (selection.kind !== "section" && selection.kind !== "field")) return;
      const tabIndex = selection.tabIndex;
      const sectionIndex = selection.sectionIndex;
      updateTabs((current) =>
        current.map((tab, ti) => {
          if (ti !== tabIndex) return tab;
          return {
            ...tab,
            sections: tab.sections.map((section, si) => {
              if (si !== sectionIndex) return section;
              return {
                ...section,
                subgrids: section.subgrids.filter((_, i) => i !== subgridIndex),
              };
            }),
          };
        }),
      );
    },
    [selectedSection, selection, updateTabs],
  );

  return {
    tabs,
    activeTabIndex,
    activeTab,
    selection,
    activeDropLineId,
    dragLabel,
    placedFieldNames,
    canUndo: history.length > 0,
    canRedo: future.length > 0,

    setActiveTabIndex,
    setSelection,
    setActiveDropLineId,
    setDragLabel,

    updateTabs,
    undo,
    redo,
    addTab,
    addSectionToActiveTab,
    addFieldToSection,
    placeFieldInSection,
    deleteField,
    handleCanvasKeyDown,
    resetFromTabs,

    selectedTab,
    selectedSection,
    selectedField,
    updateSelectedTab,
    updateSelectedSection,
    updateSelectedField,
    addSubgridToSelectedSection,
    updateSubgridInSelectedSection,
    removeSubgridFromSelectedSection,
  };
}
