import { useCallback } from "react";

import type {
  FormFieldPlacement,
  FormSection,
  FormSubgrid,
  FormTab,
  SelectionState,
} from "@/components/entities/forms/form-designer/types";

type UseFormDesignerPropertiesArgs = {
  selection: SelectionState;
  selectedTab: FormTab | null;
  selectedSection: FormSection | null;
  selectedField: FormFieldPlacement | null;
  updateTabs: (mutator: (current: FormTab[]) => FormTab[]) => void;
};

export function useFormDesignerProperties({
  selection,
  selectedTab,
  selectedSection,
  selectedField,
  updateTabs,
}: UseFormDesignerPropertiesArgs) {
  const updateSelectedTab = useCallback(
    (patch: Partial<FormTab>): void => {
      if (!selectedTab || selection.kind !== "tab") {
        return;
      }
      const tabIndex = selection.tabIndex;
      updateTabs((current) =>
        current.map((tab, currentTabIndex) =>
          currentTabIndex === tabIndex ? { ...tab, ...patch } : tab,
        ),
      );
    },
    [selectedTab, selection, updateTabs],
  );

  const updateSelectedSection = useCallback(
    (patch: Partial<FormSection>): void => {
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
    },
    [selectedSection, selection, updateTabs],
  );

  const updateSelectedField = useCallback(
    (patch: Partial<FormFieldPlacement>): void => {
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
    },
    [selectedField, selection, updateTabs],
  );

  const addSubgridToSelectedSection = useCallback((): void => {
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
  }, [selectedSection, selection, updateTabs]);

  const updateSubgridInSelectedSection = useCallback(
    (subgridIndex: number, patch: Partial<FormSubgrid>): void => {
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
    },
    [selectedSection, selection, updateTabs],
  );

  const removeSubgridFromSelectedSection = useCallback(
    (subgridIndex: number): void => {
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
                subgrids: section.subgrids.filter(
                  (_, currentSubgridIndex) => currentSubgridIndex !== subgridIndex,
                ),
              };
            }),
          };
        }),
      );
    },
    [selectedSection, selection, updateTabs],
  );

  return {
    updateSelectedTab,
    updateSelectedSection,
    updateSelectedField,
    addSubgridToSelectedSection,
    updateSubgridInSelectedSection,
    removeSubgridFromSelectedSection,
  };
}
