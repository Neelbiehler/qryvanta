"use client";

import { type ReactNode, useEffect, useState } from "react";

import { apiFetch, type RuntimeRecordResponse } from "@/lib/api";

type FormPreviewDataProviderProps = {
  entityLogicalName: string;
  enabled: boolean;
  children: (data: {
    values: Record<string, unknown>;
    isLoading: boolean;
    errorMessage: string | null;
  }) => ReactNode;
};

export function FormPreviewDataProvider({
  entityLogicalName,
  enabled,
  children,
}: FormPreviewDataProviderProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [values, setValues] = useState<Record<string, unknown>>({});

  useEffect(() => {
    if (!enabled) {
      setIsLoading(false);
      setErrorMessage(null);
      setValues({});
      return;
    }

    let isMounted = true;

    async function loadSampleRecord(): Promise<void> {
      setIsLoading(true);
      setErrorMessage(null);
      try {
        const response = await apiFetch(
          `/api/runtime/${encodeURIComponent(entityLogicalName)}/records?limit=1&offset=0`,
        );
        if (!response.ok) {
          if (!isMounted) return;
          setErrorMessage("No sample records available.");
          setValues({});
          return;
        }

        const records = (await response.json()) as RuntimeRecordResponse[];
        if (!isMounted) return;

        if (records.length === 0) {
          setErrorMessage("No sample records available.");
          setValues({});
          return;
        }

        setValues(records[0].data);
      } catch {
        if (!isMounted) return;
        setErrorMessage("Unable to load sample data.");
        setValues({});
      } finally {
        if (isMounted) {
          setIsLoading(false);
        }
      }
    }

    void loadSampleRecord();

    return () => {
      isMounted = false;
    };
  }, [enabled, entityLogicalName]);

  return children({ values, isLoading, errorMessage });
}
