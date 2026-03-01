import { useCallback, useEffect, useRef, useState } from "react";

import { apiFetch, type EntityResponse, type PublishedSchemaResponse } from "@/lib/api";

export function useRuntimeSchemas(triggerType: string, triggerEntityLogicalName: string) {
  const schemaRequestInFlightRef = useRef(new Set<string>());
  const [runtimeEntities, setRuntimeEntities] = useState<EntityResponse[]>([]);
  const [publishedSchemasByEntity, setPublishedSchemasByEntity] = useState<
    Record<string, PublishedSchemaResponse>
  >({});

  const loadPublishedSchemaForEntity = useCallback(
    async (entityLogicalName: string) => {
      const normalized = entityLogicalName.trim();
      if (normalized.length === 0) {
        return;
      }

      if (publishedSchemasByEntity[normalized]) {
        return;
      }

      if (schemaRequestInFlightRef.current.has(normalized)) {
        return;
      }

      schemaRequestInFlightRef.current.add(normalized);
      try {
        const response = await apiFetch(
          `/api/entities/${encodeURIComponent(normalized)}/published`,
        );
        if (!response.ok) {
          return;
        }

        const schema = (await response.json()) as PublishedSchemaResponse;
        setPublishedSchemasByEntity((current) => {
          if (current[normalized]) {
            return current;
          }

          return {
            ...current,
            [normalized]: schema,
          };
        });
      } finally {
        schemaRequestInFlightRef.current.delete(normalized);
      }
    },
    [publishedSchemasByEntity],
  );

  useEffect(() => {
    let active = true;

    async function loadRuntimeEntities() {
      const response = await apiFetch("/api/entities");
      if (!response.ok || !active) {
        return;
      }

      const payload = (await response.json()) as EntityResponse[];
      if (!active) {
        return;
      }

      setRuntimeEntities(payload);
    }

    void loadRuntimeEntities();
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    for (const entity of runtimeEntities) {
      void loadPublishedSchemaForEntity(entity.logical_name);
    }
  }, [loadPublishedSchemaForEntity, runtimeEntities]);

  useEffect(() => {
    if (triggerType === "manual" || triggerType === "schedule_tick") {
      return;
    }

    void loadPublishedSchemaForEntity(triggerEntityLogicalName);
  }, [loadPublishedSchemaForEntity, triggerEntityLogicalName, triggerType]);

  return {
    runtimeEntities,
    publishedSchemasByEntity,
    loadPublishedSchemaForEntity,
  };
}
