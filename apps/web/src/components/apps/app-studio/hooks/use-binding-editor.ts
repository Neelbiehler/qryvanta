import { useState } from "react";

import type {
  BindingDraft,
} from "@/components/apps/app-studio/sections";
import type { EntityResponse, FieldResponse } from "@/lib/api";

type UseBindingEditorInput = {
  entities: EntityResponse[];
};

export function useBindingEditor({ entities }: UseBindingEditorInput) {
  const [bindingDraft, setBindingDraft] = useState<BindingDraft>({
    entityToBind: entities.at(0)?.logical_name ?? "",
    navigationLabel: "",
    navigationOrder: 0,
    forms: [
      {
        logicalName: "main_form",
        displayName: "Main Form",
        fieldLogicalNames: [],
      },
    ],
    listViews: [
      {
        logicalName: "main_view",
        displayName: "Main View",
        fieldLogicalNames: [],
      },
    ],
    defaultFormLogicalName: "main_form",
    defaultListViewLogicalName: "main_view",
    defaultViewMode: "grid",
  });
  const [selectedEntityFields, setSelectedEntityFields] = useState<FieldResponse[]>([]);
  const [isLoadingEntityFields, setIsLoadingEntityFields] = useState(false);

  return {
    bindingDraft,
    setBindingDraft,
    selectedEntityFields,
    setSelectedEntityFields,
    isLoadingEntityFields,
    setIsLoadingEntityFields,
  };
}
