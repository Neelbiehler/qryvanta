"use client";

import { useEffect, useState } from "react";

const SIDEBAR_STORAGE_KEY = "sidebar-collapsed";

export function useSidebarState(storageKey: string = SIDEBAR_STORAGE_KEY) {
  const [collapsed, setCollapsed] = useState(() => {
    if (typeof window === "undefined") {
      return false;
    }

    return localStorage.getItem(storageKey) === "true";
  });

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    localStorage.setItem(storageKey, String(collapsed));
  }, [collapsed, storageKey]);

  return {
    collapsed,
    setCollapsed,
    toggleCollapsed: () => setCollapsed((current) => !current),
  };
}
