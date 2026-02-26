import { useState } from "react";

import type { NewAppDraft } from "@/components/apps/app-studio/sections";

export function useAppCatalog() {
  const [newAppDraft, setNewAppDraft] = useState<NewAppDraft>({
    logicalName: "",
    displayName: "",
    description: "",
  });

  return {
    newAppDraft,
    setNewAppDraft,
  };
}
